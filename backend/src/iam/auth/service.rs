use std::sync::Arc;

use crate::error::AppError;
use crate::iam::role::RoleRepository;
use crate::iam::user::{OAuthProvider, PasswordService, User, UserError, UserRepository};
use crate::kernel::UserId;

use super::config::AuthConfig;
use super::error::AuthError;
use super::model::{LoginMetadata, LoginRequest, OAuthUserInfo, RefreshToken, TokenClaims, TokenPair, UserSession};
use super::port::{SessionRepository, TokenRepository, TokenService};

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    password_svc: Arc<dyn PasswordService>,
    token_svc: Arc<dyn TokenService>,
    token_repo: Arc<dyn TokenRepository>,
    session_repo: Arc<dyn SessionRepository>,
    role_repo: Arc<dyn RoleRepository>,
    config: AuthConfig,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        password_svc: Arc<dyn PasswordService>,
        token_svc: Arc<dyn TokenService>,
        token_repo: Arc<dyn TokenRepository>,
        session_repo: Arc<dyn SessionRepository>,
        role_repo: Arc<dyn RoleRepository>,
        config: AuthConfig,
    ) -> Self {
        Self {
            user_repo,
            password_svc,
            token_svc,
            token_repo,
            session_repo,
            role_repo,
            config,
        }
    }

    // ========================================================================
    // Login
    // ========================================================================

    pub async fn login_with_password(&self, req: LoginRequest, meta: LoginMetadata) -> Result<TokenPair, AppError> {
        let user = self
            .user_repo
            .get_by_email(&req.email)
            .await?
            .ok_or_else(|| AuthError::invalid_credentials())?;

        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or_else(|| AuthError::invalid_credentials())?;

        let valid = self
            .password_svc
            .verify_password(&req.password, password_hash)?;
        if !valid {
            return Err(AuthError::invalid_credentials());
        }

        if !user.can_login() {
            return Err(AuthError::account_disabled());
        }

        self.create_session_and_tokens(&user, meta).await
    }

    pub async fn find_or_create_oauth_user(
        &self,
        user_info: OAuthUserInfo,
        provider: OAuthProvider,
        meta: LoginMetadata,
    ) -> Result<TokenPair, AppError> {
        let user = match self.user_repo.get_by_email(&user_info.email).await? {
            Some(mut existing) => {
                if existing.oauth_provider.as_ref() != Some(&provider)
                    || existing.oauth_provider_id.as_deref() != Some(&user_info.id)
                {
                    existing.link_oauth(provider, user_info.id);
                    existing.update_profile(Some(user_info.name), user_info.picture);
                    self.user_repo.update(&existing).await?;
                }
                existing
            }
            None => {
                let new_user = User::new_with_oauth(
                    user_info.email,
                    user_info.name,
                    user_info.picture,
                    provider,
                    user_info.id,
                );
                self.user_repo.save(&new_user).await?;
                new_user
            }
        };

        if !user.can_login() {
            return Err(AuthError::account_disabled());
        }

        self.create_session_and_tokens(&user, meta).await
    }

    // ========================================================================
    // Token Refresh — rotates within the same session
    // ========================================================================

    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenPair, AppError> {
        // Validate the JWT signature/expiry
        let user_id = self
            .token_svc
            .validate_refresh_token(refresh_token)
            .map_err(|_| AuthError::invalid_refresh_token())?;

        // Check the DB record
        let stored = self
            .token_repo
            .get_refresh_token(refresh_token)
            .await?
            .ok_or_else(|| AuthError::invalid_refresh_token())?;

        if !stored.is_valid() {
            return Err(if stored.is_revoked {
                AuthError::invalid_refresh_token()
            } else {
                AuthError::expired_refresh_token()
            });
        }

        // Revoke old token
        self.token_repo.revoke_refresh_token(refresh_token).await?;

        // Update session activity
        self.session_repo.update_activity(&stored.session_id).await?;

        // Get user and generate new token pair within the same session
        let user = self
            .user_repo
            .get_by_id(&user_id)
            .await?
            .ok_or_else(|| UserError::not_found())?;

        if !user.can_login() {
            return Err(AuthError::account_disabled());
        }

        self.generate_token_pair(&user, &stored.session_id).await
    }

    // ========================================================================
    // Logout
    // ========================================================================

    /// Logout a single session — revokes the session and all its refresh tokens
    pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        // Find the token to get its session_id
        if let Some(stored) = self.token_repo.get_refresh_token(refresh_token).await? {
            self.token_repo.revoke_by_session(&stored.session_id).await?;
            self.session_repo.revoke(&stored.session_id).await?;
        }
        Ok(())
    }

    /// Logout all sessions — revokes all tokens and sessions for the user
    pub async fn logout_all(&self, user_id: &UserId) -> Result<(), AppError> {
        self.token_repo.revoke_all_for_user(user_id).await?;
        self.session_repo.revoke_all_for_user(user_id).await
    }

    // ========================================================================
    // Session Management
    // ========================================================================

    pub async fn list_user_sessions(&self, user_id: &UserId) -> Result<Vec<UserSession>, AppError> {
        self.session_repo.list_by_user(user_id).await
    }

    /// Revoke a specific session by ID — also revokes all its refresh tokens
    pub async fn revoke_session(&self, session_id: &str) -> Result<(), AppError> {
        self.token_repo.revoke_by_session(session_id).await?;
        self.session_repo.revoke(session_id).await
    }

    pub async fn clean_expired_sessions(&self) -> Result<(), AppError> {
        self.session_repo.clean_expired().await
    }

    // ========================================================================
    // Private
    // ========================================================================

    /// Creates a new session + first token pair (used on login)
    async fn create_session_and_tokens(
        &self,
        user: &User,
        meta: LoginMetadata,
    ) -> Result<TokenPair, AppError> {
        let session = UserSession::new(
            user.id.clone(),
            meta.ip_address,
            meta.user_agent,
            self.config.jwt.refresh_token_ttl,
        );
        self.session_repo.save(&session).await?;

        self.generate_token_pair(user, &session.id).await
    }

    /// Generates access + refresh tokens linked to an existing session
    async fn generate_token_pair(&self, user: &User, session_id: &str) -> Result<TokenPair, AppError> {
        let scopes = self.resolve_effective_scopes(&user.id).await?;

        let claims = TokenClaims {
            user_id: user.id.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            scopes,
        };

        let access_token = self.token_svc.generate_access_token(&claims)?;
        let refresh_token_str = self.token_svc.generate_refresh_token(&user.id)?;

        let refresh_token = RefreshToken::new(
            user.id.clone(),
            session_id.to_string(),
            refresh_token_str.clone(),
            self.config.jwt.refresh_token_ttl,
        );
        self.token_repo.save_refresh_token(&refresh_token).await?;

        Ok(TokenPair {
            access_token,
            refresh_token: refresh_token_str,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt.access_token_ttl.as_secs() as i64,
        })
    }

    async fn resolve_effective_scopes(&self, user_id: &UserId) -> Result<Vec<String>, AppError> {
        let roles = self.role_repo.list_by_user(user_id).await?;
        let mut seen = std::collections::HashSet::new();
        let mut scopes = Vec::new();
        for role in &roles {
            for scope in &role.scopes {
                if seen.insert(scope.clone()) {
                    scopes.push(scope.clone());
                }
            }
        }
        Ok(scopes)
    }
}
