use crate::error::AppError;
use crate::kernel::{MovieId, Paginated, PaginationOptions, UserId};

use super::model::{Movie, MovieFilter};

#[async_trait::async_trait]
pub trait MovieRepository: Send + Sync {
    async fn save(&self, movie: &Movie) -> Result<(), AppError>;
    async fn get_by_id(&self, id: &MovieId) -> Result<Option<Movie>, AppError>;
    async fn find(
        &self,
        opts: &PaginationOptions,
        filter: &MovieFilter,
    ) -> Result<Paginated<Movie>, AppError>;
    async fn update(&self, movie: &Movie) -> Result<(), AppError>;
    async fn delete(&self, id: &MovieId) -> Result<(), AppError>;
    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Movie>, AppError>;
}
