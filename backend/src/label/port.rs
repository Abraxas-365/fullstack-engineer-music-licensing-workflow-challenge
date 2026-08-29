use crate::error::AppError;
use crate::kernel::{LabelId, UserId};

use super::model::{Label, LabelMember};

#[async_trait::async_trait]
pub trait LabelRepository: Send + Sync {
    async fn save(&self, label: &Label) -> Result<(), AppError>;
    async fn get_by_id(&self, id: &LabelId) -> Result<Option<Label>, AppError>;
    async fn get_by_name(&self, name: &str) -> Result<Option<Label>, AppError>;
    async fn list_all(&self) -> Result<Vec<Label>, AppError>;
    async fn update(&self, label: &Label) -> Result<(), AppError>;
    async fn delete(&self, id: &LabelId) -> Result<(), AppError>;

    // Membership
    async fn add_member(&self, member: &LabelMember) -> Result<(), AppError>;
    async fn remove_member(&self, label_id: &LabelId, user_id: &UserId) -> Result<(), AppError>;
    async fn get_member(
        &self,
        label_id: &LabelId,
        user_id: &UserId,
    ) -> Result<Option<LabelMember>, AppError>;
    async fn list_members(&self, label_id: &LabelId) -> Result<Vec<LabelMember>, AppError>;
    async fn get_user_labels(&self, user_id: &UserId) -> Result<Vec<Label>, AppError>;
}
