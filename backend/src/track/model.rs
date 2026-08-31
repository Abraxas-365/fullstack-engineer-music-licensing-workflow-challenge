use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::AppError;
use crate::kernel::{SceneId, SongId, TrackId, UserId};

// ============================================================================
// Entity
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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
    /// Start of the excerpt within the song's own timeline, in seconds.
    pub start_time_seconds: i32,
    /// End of the excerpt within the song's own timeline, in seconds.
    pub end_time_seconds: i32,
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
        start_time_seconds: i32,
        end_time_seconds: i32,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: TrackId::new(),
            scene_id,
            song_id,
            usage_type,
            start_time_seconds,
            end_time_seconds,
            created_by,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn duration_seconds(&self) -> i32 {
        self.end_time_seconds - self.start_time_seconds
    }
}

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTrackRequest {
    pub scene_id: SceneId,
    pub song_id: SongId,
    pub usage_type: String,
    /// Start of the excerpt within the song's own timeline, in seconds.
    pub start_time_seconds: i32,
    /// End of the excerpt within the song's own timeline, in seconds.
    pub end_time_seconds: i32,
    pub notes: Option<String>,
}

impl CreateTrackRequest {
    pub fn validate(&self) -> Result<UsageType, AppError> {
        if self.start_time_seconds < 0 {
            return Err(AppError::validation("Start time cannot be negative")
                .with_detail("field", "start_time_seconds"));
        }
        if self.end_time_seconds <= self.start_time_seconds {
            return Err(
                AppError::validation("End time must be greater than start time")
                    .with_detail("field", "end_time_seconds"),
            );
        }
        UsageType::try_from(self.usage_type.as_str())
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTrackRequest {
    pub usage_type: Option<String>,
    pub start_time_seconds: Option<i32>,
    pub end_time_seconds: Option<i32>,
    pub notes: Option<String>,
}

impl UpdateTrackRequest {
    pub fn validate(&self) -> Result<Option<UsageType>, AppError> {
        if let Some(start) = self.start_time_seconds {
            if start < 0 {
                return Err(AppError::validation("Start time cannot be negative")
                    .with_detail("field", "start_time_seconds"));
            }
        }
        match &self.usage_type {
            Some(ut) => Ok(Some(UsageType::try_from(ut.as_str())?)),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TrackResponse {
    pub id: TrackId,
    pub scene_id: SceneId,
    pub song_id: SongId,
    pub usage_type: String,
    pub start_time_seconds: i32,
    pub end_time_seconds: i32,
    pub duration_seconds: i32,
    pub created_by: UserId,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Track> for TrackResponse {
    fn from(t: Track) -> Self {
        let duration_seconds = t.duration_seconds();
        Self {
            id: t.id,
            scene_id: t.scene_id,
            song_id: t.song_id,
            usage_type: t.usage_type.as_str().to_string(),
            start_time_seconds: t.start_time_seconds,
            end_time_seconds: t.end_time_seconds,
            duration_seconds,
            created_by: t.created_by,
            notes: t.notes,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}
