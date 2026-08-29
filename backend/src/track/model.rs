use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::kernel::{SceneId, SongId, TrackId, UserId};

// ============================================================================
// Entity
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsageType {
    Background,
    Featured,
    Credits,
    Trailer,
}

impl UsageType {
    pub fn as_str(&self) -> &str {
        match self {
            UsageType::Background => "BACKGROUND",
            UsageType::Featured => "FEATURED",
            UsageType::Credits => "CREDITS",
            UsageType::Trailer => "TRAILER",
        }
    }
}

impl TryFrom<&str> for UsageType {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "BACKGROUND" => Ok(UsageType::Background),
            "FEATURED" => Ok(UsageType::Featured),
            "CREDITS" => Ok(UsageType::Credits),
            "TRAILER" => Ok(UsageType::Trailer),
            _ => Err(AppError::validation(format!("Invalid usage type: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub scene_id: SceneId,
    pub song_id: SongId,
    pub usage_type: UsageType,
    pub created_by: UserId,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Track {
    pub fn new(
        scene_id: SceneId,
        song_id: SongId,
        usage_type: UsageType,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: TrackId::new(),
            scene_id,
            song_id,
            usage_type,
            created_by,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }
}

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTrackRequest {
    pub scene_id: SceneId,
    pub song_id: SongId,
    pub usage_type: String,
    pub notes: Option<String>,
}

impl CreateTrackRequest {
    pub fn validate(&self) -> Result<UsageType, AppError> {
        UsageType::try_from(self.usage_type.as_str())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackRequest {
    pub usage_type: Option<String>,
    pub notes: Option<String>,
}

impl UpdateTrackRequest {
    pub fn validate(&self) -> Result<Option<UsageType>, AppError> {
        match &self.usage_type {
            Some(ut) => Ok(Some(UsageType::try_from(ut.as_str())?)),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackResponse {
    pub id: TrackId,
    pub scene_id: SceneId,
    pub song_id: SongId,
    pub usage_type: String,
    pub created_by: UserId,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Track> for TrackResponse {
    fn from(t: Track) -> Self {
        Self {
            id: t.id,
            scene_id: t.scene_id,
            song_id: t.song_id,
            usage_type: t.usage_type.as_str().to_string(),
            created_by: t.created_by,
            notes: t.notes,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}
