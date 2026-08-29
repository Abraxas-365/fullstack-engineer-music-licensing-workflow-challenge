use super::User;
use crate::error::AppError;
use crate::kernel::{Paginated, PaginationOptions, UserId};

use super::model::UserFilter;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_by_id(&self, id: &UserId) -> Result<Option<User>, AppError>;
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn find(
        &self,
        opts: &PaginationOptions,
        filter: &UserFilter,
    ) -> Result<Paginated<User>, AppError>;
    async fn save(&self, user: &User) -> Result<(), AppError>;
    async fn update(&self, user: &User) -> Result<(), AppError>;
    async fn delete(&self, id: &UserId) -> Result<(), AppError>;
}

pub trait PasswordService: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, AppError>;
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError>;
}
