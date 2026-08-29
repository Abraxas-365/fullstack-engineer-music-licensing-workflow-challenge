use std::sync::Arc;

use crate::error::AppError;
use crate::kernel::{Paginated, PaginationOptions, UserId};

use super::{
    CreateUserRequest, UpdateUserRequest, User, UserError, UserRepository, UserResponse,
    PasswordService,
};
use super::model::UserFilter;

pub struct UserService {
    repo: Arc<dyn UserRepository>,
    password_svc: Arc<dyn PasswordService>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>, password_svc: Arc<dyn PasswordService>) -> Self {
        Self { repo, password_svc }
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> Result<UserResponse, AppError> {
        req.validate()?;

        if self.repo.get_by_email(&req.email).await?.is_some() {
            return Err(UserError::already_exists());
        }

        let password_hash = self.password_svc.hash_password(&req.password)?;
        let user = User::new_with_password(req.email, req.name, password_hash);
        self.repo.save(&user).await?;

        Ok(user.into())
    }

    pub async fn get_user(&self, id: &UserId) -> Result<UserResponse, AppError> {
        let user = self.repo.get_by_id(id).await?
            .ok_or_else(|| UserError::not_found())?;
        Ok(user.into())
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<UserResponse, AppError> {
        let user = self.repo.get_by_email(email).await?
            .ok_or_else(|| UserError::not_found())?;
        Ok(user.into())
    }

    pub async fn find_users(
        &self,
        opts: &PaginationOptions,
        filter: &UserFilter,
    ) -> Result<Paginated<UserResponse>, AppError> {
        let page = self.repo.find(opts, filter).await?;
        let responses = page.items.into_iter().map(UserResponse::from).collect();
        Ok(Paginated::new(
            responses,
            page.pagination.page,
            page.pagination.page_size,
            page.pagination.total,
        ))
    }

    pub async fn update_user(
        &self,
        id: &UserId,
        req: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        req.validate()?;

        let mut user = self.repo.get_by_id(id).await?
            .ok_or_else(|| UserError::not_found())?;

        if let Some(name) = req.name {
            user.update_profile(Some(name), None);
        }

        if let Some(status) = req.status {
            match status {
                super::UserStatus::Active => user.activate()?,
                super::UserStatus::Suspended => user.suspend("Updated by admin")?,
                _ => return Err(UserError::invalid_status()),
            }
        }

        self.repo.update(&user).await?;
        Ok(user.into())
    }

    pub async fn activate_user(&self, id: &UserId) -> Result<UserResponse, AppError> {
        let mut user = self.repo.get_by_id(id).await?
            .ok_or_else(|| UserError::not_found())?;
        user.activate()?;
        self.repo.update(&user).await?;
        Ok(user.into())
    }

    pub async fn suspend_user(&self, id: &UserId, reason: &str) -> Result<UserResponse, AppError> {
        let mut user = self.repo.get_by_id(id).await?
            .ok_or_else(|| UserError::not_found())?;
        user.suspend(reason)?;
        self.repo.update(&user).await?;
        Ok(user.into())
    }

    pub async fn delete_user(&self, id: &UserId) -> Result<(), AppError> {
        self.repo.get_by_id(id).await?
            .ok_or_else(|| UserError::not_found())?;
        self.repo.delete(id).await
    }
}
