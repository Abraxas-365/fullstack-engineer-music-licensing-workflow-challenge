use sqlx::PgPool;

use crate::error::AppError;
use crate::kernel::{MovieId, Paginated, PaginationOptions, UserId};

use super::super::model::{Movie, MovieFilter};
use super::super::port::MovieRepository;

const MOVIE_COLUMNS: &str =
    "id, title, description, release_year, director, created_by, created_at, updated_at";

pub struct PostgresMovieRepository {
    pool: PgPool,
}

impl PostgresMovieRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn movie_from_row(row: &sqlx::postgres::PgRow) -> Result<Movie, AppError> {
    use sqlx::Row;
    Ok(Movie {
        id: MovieId::from_string(
            row.try_get("id")
                .map_err(|e| AppError::internal(e.to_string()))?,
        ),
        title: row
            .try_get("title")
            .map_err(|e| AppError::internal(e.to_string()))?,
        description: row
            .try_get("description")
            .map_err(|e| AppError::internal(e.to_string()))?,
        release_year: row
            .try_get("release_year")
            .map_err(|e| AppError::internal(e.to_string()))?,
        director: row
            .try_get("director")
            .map_err(|e| AppError::internal(e.to_string()))?,
        created_by: UserId::from_string(
            row.try_get("created_by")
                .map_err(|e| AppError::internal(e.to_string()))?,
        ),
        created_at: row
            .try_get("created_at")
            .map_err(|e| AppError::internal(e.to_string()))?,
        updated_at: row
            .try_get("updated_at")
            .map_err(|e| AppError::internal(e.to_string()))?,
    })
}

#[async_trait::async_trait]
impl MovieRepository for PostgresMovieRepository {
    async fn save(&self, movie: &Movie) -> Result<(), AppError> {
        sqlx::query(&format!(
            "INSERT INTO movies ({MOVIE_COLUMNS}) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        ))
        .bind(movie.id.as_str())
        .bind(&movie.title)
        .bind(&movie.description)
        .bind(movie.release_year)
        .bind(&movie.director)
        .bind(movie.created_by.as_str())
        .bind(movie.created_at)
        .bind(movie.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;
        Ok(())
    }

    async fn get_by_id(&self, id: &MovieId) -> Result<Option<Movie>, AppError> {
        let row = sqlx::query(&format!("SELECT {MOVIE_COLUMNS} FROM movies WHERE id = $1"))
            .bind(id.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::internal(e.to_string()))?;

        row.as_ref().map(movie_from_row).transpose()
    }

    async fn find(
        &self,
        opts: &PaginationOptions,
        filter: &MovieFilter,
    ) -> Result<Paginated<Movie>, AppError> {
        let count_row = sqlx::query(
            "SELECT COUNT(*) as count FROM movies
             WHERE ($1::text IS NULL OR title ILIKE '%' || $1 || '%')
             AND ($2::text IS NULL OR created_by = $2)",
        )
        .bind(filter.search.as_deref())
        .bind(filter.created_by.as_ref().map(|id| id.as_str()))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

        let total: i64 = sqlx::Row::try_get(&count_row, "count")
            .map_err(|e| AppError::internal(e.to_string()))?;

        let rows = sqlx::query(&format!(
            "SELECT {MOVIE_COLUMNS} FROM movies
             WHERE ($1::text IS NULL OR title ILIKE '%' || $1 || '%')
             AND ($2::text IS NULL OR created_by = $2)
             ORDER BY created_at DESC
             LIMIT $3 OFFSET $4"
        ))
        .bind(filter.search.as_deref())
        .bind(filter.created_by.as_ref().map(|id| id.as_str()))
        .bind(opts.limit())
        .bind(opts.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

        let items: Result<Vec<Movie>, AppError> = rows.iter().map(movie_from_row).collect();

        Ok(Paginated::new(items?, opts.page, opts.page_size, total))
    }

    async fn update(&self, movie: &Movie) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE movies SET title = $1, description = $2, release_year = $3, director = $4, updated_at = $5 WHERE id = $6",
        )
        .bind(&movie.title)
        .bind(&movie.description)
        .bind(movie.release_year)
        .bind(&movie.director)
        .bind(movie.updated_at)
        .bind(movie.id.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, id: &MovieId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM movies WHERE id = $1")
            .bind(id.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(e.to_string()))?;
        Ok(())
    }

    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<Movie>, AppError> {
        let rows = sqlx::query(&format!(
            "SELECT {MOVIE_COLUMNS} FROM movies WHERE created_by = $1 ORDER BY created_at DESC"
        ))
        .bind(user_id.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(e.to_string()))?;

        rows.iter().map(movie_from_row).collect()
    }
}
