use std::sync::Arc;

use crate::error::AppError;
use crate::iam::scopes;
use crate::iam::user::{UserError, UserRepository};
use crate::kernel::{RoleId, UserId};

use super::{
    CreateRoleRequest, Role, RoleError, RoleRepository, RoleResponse, UpdateRoleRequest,
    UserRolesResponse,
};

pub struct RoleService {
    repo: Arc<dyn RoleRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl RoleService {
    pub fn new(repo: Arc<dyn RoleRepository>, user_repo: Arc<dyn UserRepository>) -> Self {
        Self { repo, user_repo }
    }

    pub async fn create_role(&self, req: CreateRoleRequest) -> Result<RoleResponse, AppError> {
        req.validate()?;
        validate_scopes(&req.scopes)?;

        // Check for duplicate name
        if self.repo.get_by_name(&req.name).await?.is_some() {
            return Err(RoleError::already_exists().with_detail("name", req.name));
        }

        let role = Role::new(req.name, req.description.unwrap_or_default(), req.scopes);

        self.repo.save(&role).await?;
        Ok(RoleResponse::from(&role))
    }

    pub async fn get_role(&self, id: &RoleId) -> Result<RoleResponse, AppError> {
        let role = self
            .repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| RoleError::not_found())?;
        Ok(RoleResponse::from(&role))
    }

    pub async fn list_roles(&self) -> Result<Vec<RoleResponse>, AppError> {
        let roles = self.repo.list_all().await?;
        Ok(roles.iter().map(RoleResponse::from).collect())
    }

    pub async fn update_role(
        &self,
        id: &RoleId,
        req: UpdateRoleRequest,
    ) -> Result<RoleResponse, AppError> {
        req.validate()?;
        let mut role = self
            .repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| RoleError::not_found())?;

        if let Some(name) = req.name {
            // Check for duplicate name (exclude current role)
            if let Some(existing) = self.repo.get_by_name(&name).await? {
                if existing.id != role.id {
                    return Err(RoleError::already_exists().with_detail("name", name));
                }
            }
            role.name = name;
        }

        if let Some(description) = req.description {
            role.description = description;
        }

        if let Some(new_scopes) = req.scopes {
            validate_scopes(&new_scopes)?;
            role.set_scopes(new_scopes);
        }

        role.updated_at = chrono::Utc::now();
        self.repo.save(&role).await?;
        Ok(RoleResponse::from(&role))
    }

    pub async fn delete_role(&self, id: &RoleId) -> Result<(), AppError> {
        self.repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| RoleError::not_found())?;
        self.repo.delete(id).await
    }

    pub async fn assign_role_to_user(
        &self,
        role_id: &RoleId,
        user_id: &UserId,
    ) -> Result<(), AppError> {
        self.repo
            .get_by_id(role_id)
            .await?
            .ok_or_else(|| RoleError::not_found())?;

        self.user_repo
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| UserError::not_found())?;

        self.repo.assign_to_user(user_id, role_id).await
    }

    pub async fn unassign_role_from_user(
        &self,
        role_id: &RoleId,
        user_id: &UserId,
    ) -> Result<(), AppError> {
        self.repo.unassign_from_user(user_id, role_id).await
    }

    pub async fn get_user_roles(&self, user_id: &UserId) -> Result<UserRolesResponse, AppError> {
        self.user_repo
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| UserError::not_found())?;

        let roles = self.repo.list_by_user(user_id).await?;
        let effective_scopes = resolve_effective_scopes(&roles);
        let role_dtos = roles.iter().map(RoleResponse::from).collect();

        Ok(UserRolesResponse {
            user_id: user_id.clone(),
            roles: role_dtos,
            effective_scopes,
        })
    }

    pub async fn get_effective_scopes(&self, user_id: &UserId) -> Result<Vec<String>, AppError> {
        self.user_repo
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| UserError::not_found())?;

        let roles = self.repo.list_by_user(user_id).await?;
        Ok(resolve_effective_scopes(&roles))
    }
}

fn validate_scopes(scope_list: &[String]) -> Result<(), AppError> {
    if scope_list.is_empty() {
        return Err(
            RoleError::invalid_scopes().with_detail("reason", "at least one scope is required")
        );
    }

    let invalid: Vec<&String> = scope_list
        .iter()
        .filter(|s| !scopes::validate_scope(s))
        .collect();

    if !invalid.is_empty() {
        let names: Vec<&str> = invalid.iter().map(|s| s.as_str()).collect();
        return Err(RoleError::invalid_scopes()
            .with_detail("invalid_scopes", serde_json::json!(names))
            .with_detail("hint", "Use the scope catalog to see valid options"));
    }

    Ok(())
}

fn resolve_effective_scopes(roles: &[Role]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for role in roles {
        for scope in &role.scopes {
            if seen.insert(scope.clone()) {
                result.push(scope.clone());
            }
        }
    }

    result
}
