use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::kernel::{MovieId, UserId};

// ============================================================================
// Entity
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: MovieId,
    pub title: String,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub director: Option<String>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Movie {
    pub fn new(title: String, created_by: UserId) -> Self {
        let now = Utc::now();
        Self {
            id: MovieId::new(),
            title,
            description: None,
            release_year: None,
            director: None,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }
}

// ============================================================================
// Movie Membership
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovieRole {
    Owner,
    Supervisor,
    Editor,
    Viewer,
}

impl MovieRole {
    pub fn as_str(&self) -> &str {
        match self {
            MovieRole::Owner => "OWNER",
            MovieRole::Supervisor => "SUPERVISOR",
            MovieRole::Editor => "EDITOR",
            MovieRole::Viewer => "VIEWER",
        }
    }
}

impl TryFrom<&str> for MovieRole {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "OWNER" => Ok(MovieRole::Owner),
            "SUPERVISOR" => Ok(MovieRole::Supervisor),
            "EDITOR" => Ok(MovieRole::Editor),
            "VIEWER" => Ok(MovieRole::Viewer),
            _ => Err(AppError::validation(format!("Invalid movie role: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieMember {
    pub movie_id: MovieId,
    pub user_id: UserId,
    pub role: MovieRole,
    pub joined_at: DateTime<Utc>,
}

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateMovieRequest {
    pub title: String,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub director: Option<String>,
}

impl CreateMovieRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.title.trim().is_empty() {
            return Err(AppError::validation("Title is required").with_detail("field", "title"));
        }
        if let Some(year) = self.release_year {
            if year < 1888 || year > 2100 {
                return Err(
                    AppError::validation("Release year must be between 1888 and 2100")
                        .with_detail("field", "release_year"),
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateMovieRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub director: Option<String>,
}

impl UpdateMovieRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(ref title) = self.title {
            if title.trim().is_empty() {
                return Err(
                    AppError::validation("Title cannot be empty").with_detail("field", "title")
                );
            }
        }
        if let Some(year) = self.release_year {
            if year < 1888 || year > 2100 {
                return Err(
                    AppError::validation("Release year must be between 1888 and 2100")
                        .with_detail("field", "release_year"),
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MovieResponse {
    pub id: MovieId,
    pub title: String,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub director: Option<String>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Movie> for MovieResponse {
    fn from(m: Movie) -> Self {
        Self {
            id: m.id,
            title: m.title,
            description: m.description,
            release_year: m.release_year,
            director: m.director,
            created_by: m.created_by,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct MovieFilter {
    pub search: Option<String>,
    pub created_by: Option<UserId>,
}

#[derive(Debug, Serialize)]
pub struct MovieMemberResponse {
    pub user_id: UserId,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

impl From<&MovieMember> for MovieMemberResponse {
    fn from(m: &MovieMember) -> Self {
        Self {
            user_id: m.user_id.clone(),
            role: m.role.as_str().to_string(),
            joined_at: m.joined_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddMovieMemberRequest {
    pub user_id: UserId,
    pub role: Option<String>,
}
