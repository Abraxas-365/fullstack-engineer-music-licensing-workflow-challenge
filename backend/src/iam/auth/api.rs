use actix_web::{HttpRequest, HttpResponse, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

use super::middleware::AuthContext;
use super::model::LoginMetadata;
use super::service::AuthService;

// ============================================================================
// Request / Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshBody {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutBody {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

// ============================================================================
// Handlers
// ============================================================================

fn extract_meta(req: &HttpRequest) -> LoginMetadata {
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    LoginMetadata {
        ip_address,
        user_agent,
    }
}

pub async fn login(
    req: HttpRequest,
    auth_svc: web::Data<AuthService>,
    body: web::Json<LoginBody>,
) -> Result<HttpResponse, AppError> {
    let meta = extract_meta(&req);
    let body = body.into_inner();

    let pair = auth_svc
        .login_with_password(
            super::model::LoginRequest {
                email: body.email,
                password: body.password,
            },
            meta,
        )
        .await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        token_type: pair.token_type,
        expires_in: pair.expires_in,
    }))
}

pub async fn refresh(
    auth_svc: web::Data<AuthService>,
    body: web::Json<RefreshBody>,
) -> Result<HttpResponse, AppError> {
    let pair = auth_svc.refresh_tokens(&body.refresh_token).await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        token_type: pair.token_type,
        expires_in: pair.expires_in,
    }))
}

pub async fn logout(
    auth_svc: web::Data<AuthService>,
    body: web::Json<LogoutBody>,
) -> Result<HttpResponse, AppError> {
    auth_svc.logout(&body.refresh_token).await?;

    Ok(HttpResponse::Ok().json(MessageResponse {
        message: "Logged out successfully".into(),
    }))
}

pub async fn logout_all(
    auth: AuthContext,
    auth_svc: web::Data<AuthService>,
) -> Result<HttpResponse, AppError> {
    auth_svc.logout_all(&auth.user_id).await?;

    Ok(HttpResponse::Ok().json(MessageResponse {
        message: "All sessions revoked".into(),
    }))
}

pub async fn me(auth: AuthContext) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(MeResponse {
        user_id: auth.user_id.to_string(),
        email: auth.email,
        name: auth.name,
        scopes: auth.scopes,
    }))
}

pub async fn list_sessions(
    auth: AuthContext,
    auth_svc: web::Data<AuthService>,
) -> Result<HttpResponse, AppError> {
    let sessions = auth_svc.list_user_sessions(&auth.user_id).await?;

    let response: Vec<SessionResponse> = sessions
        .into_iter()
        .map(|s| SessionResponse {
            id: s.id,
            ip_address: s.ip_address,
            user_agent: s.user_agent,
            created_at: s.created_at,
            last_activity: s.last_activity,
            expires_at: s.expires_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn revoke_session(
    auth: AuthContext,
    auth_svc: web::Data<AuthService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();

    // Verify the session belongs to the authenticated user
    let sessions = auth_svc.list_user_sessions(&auth.user_id).await?;
    if !sessions.iter().any(|s| s.id == session_id) {
        return Err(AppError::not_found("Session not found"));
    }

    auth_svc.revoke_session(&session_id).await?;

    Ok(HttpResponse::Ok().json(MessageResponse {
        message: "Session revoked".into(),
    }))
}

// ============================================================================
// Route Configuration
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh))
            .route("/logout", web::post().to(logout))
            .route("/logout-all", web::post().to(logout_all))
            .route("/me", web::get().to(me))
            .route("/sessions", web::get().to(list_sessions))
            .route("/sessions/{id}", web::delete().to(revoke_session)),
    );
}
