use std::sync::Arc;

use chrono::Utc;

use crate::error::AppError;
use crate::iam::user::UserRepository;
use crate::kernel::{LabelId, UserId};

use super::error::LabelError;
use super::model::{
    AddMemberRequest, CreateLabelRequest, Label, LabelMember, LabelRole, UpdateLabelRequest,
};
use super::port::LabelRepository;

pub struct LabelService {
    label_repo: Arc<dyn LabelRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl LabelService {
    pub fn new(
        label_repo: Arc<dyn LabelRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            label_repo,
            user_repo,
        }
    }

    // ========================================================================
    // Label CRUD
    // ========================================================================

    pub async fn create_label(&self, req: CreateLabelRequest) -> Result<Label, AppError> {
        req.validate()?;

        if self.label_repo.get_by_name(&req.name).await?.is_some() {
            return Err(LabelError::already_exists());
        }

        let label = Label::new(req.name, req.website, req.contact_email);
        self.label_repo.save(&label).await?;
        Ok(label)
    }

    pub async fn get_label(&self, id: &LabelId) -> Result<Label, AppError> {
        self.label_repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| LabelError::not_found())
    }

    pub async fn list_labels(&self) -> Result<Vec<Label>, AppError> {
        self.label_repo.list_all().await
    }

    pub async fn update_label(
        &self,
        id: &LabelId,
        req: UpdateLabelRequest,
    ) -> Result<Label, AppError> {
        req.validate()?;

        let mut label = self.get_label(id).await?;

        // Check name uniqueness if changing
        if let Some(ref name) = req.name {
            if name != &label.name {
                if self.label_repo.get_by_name(name).await?.is_some() {
                    return Err(LabelError::already_exists());
                }
            }
        }

        if let Some(name) = req.name {
            label.name = name;
        }
        if let Some(website) = req.website {
            label.website = Some(website);
        }
        if let Some(contact_email) = req.contact_email {
            label.contact_email = Some(contact_email);
        }
        label.updated_at = Utc::now();

        self.label_repo.update(&label).await?;
        Ok(label)
    }

    pub async fn delete_label(&self, id: &LabelId) -> Result<(), AppError> {
        self.get_label(id).await?;
        self.label_repo.delete(id).await
    }

    // ========================================================================
    // Membership
    // ========================================================================

    pub async fn add_member(
        &self,
        label_id: &LabelId,
        req: AddMemberRequest,
    ) -> Result<LabelMember, AppError> {
        // Verify label exists
        self.get_label(label_id).await?;

        // Verify user exists
        self.user_repo
            .get_by_id(&req.user_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        // Check not already a member
        if self
            .label_repo
            .get_member(label_id, &req.user_id)
            .await?
            .is_some()
        {
            return Err(LabelError::member_already_added());
        }

        let role = match req.role.as_deref() {
            Some(r) => LabelRole::try_from(r)?,
            None => LabelRole::Rep,
        };

        let member = LabelMember {
            label_id: label_id.clone(),
            user_id: req.user_id,
            role,
            joined_at: Utc::now(),
        };

        self.label_repo.add_member(&member).await?;
        Ok(member)
    }

    pub async fn remove_member(
        &self,
        label_id: &LabelId,
        user_id: &UserId,
    ) -> Result<(), AppError> {
        self.label_repo
            .get_member(label_id, user_id)
            .await?
            .ok_or_else(|| LabelError::member_not_found())?;

        self.label_repo.remove_member(label_id, user_id).await
    }

    pub async fn list_members(
        &self,
        label_id: &LabelId,
    ) -> Result<Vec<LabelMember>, AppError> {
        self.get_label(label_id).await?;
        self.label_repo.list_members(label_id).await
    }

    pub async fn get_user_labels(&self, user_id: &UserId) -> Result<Vec<Label>, AppError> {
        self.label_repo.get_user_labels(user_id).await
    }
}
