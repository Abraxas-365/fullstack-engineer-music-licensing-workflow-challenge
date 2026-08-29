use crate::error::{AppError, ErrorType};

pub struct SceneError;

impl SceneError {
    pub fn not_found() -> AppError {
        AppError::new("scene.not_found", "Scene not found", ErrorType::NotFound)
    }

    pub fn already_exists() -> AppError {
        AppError::new(
            "scene.already_exists",
            "Scene already exists",
            ErrorType::Conflict,
        )
    }
}
