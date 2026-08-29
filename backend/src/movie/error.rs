use crate::error::{AppError, ErrorType};

pub struct MovieError;

impl MovieError {
    pub fn not_found() -> AppError {
        AppError::new("movie.not_found", "Movie not found", ErrorType::NotFound)
    }

    pub fn already_exists() -> AppError {
        AppError::new(
            "movie.already_exists",
            "Movie already exists",
            ErrorType::Conflict,
        )
    }
}
