use actix_web::{HttpResponse, web};
use serde::Deserialize;

use crate::error::AppError;
use crate::iam::auth::AuthContext;
use crate::iam::scopes;
use crate::kernel::{MovieId, PaginationOptions, UserId};
use crate::scene::{SceneResponse, SceneService};

use super::model::{
    AddMovieMemberRequest, CreateMovieRequest, MovieFilter, MovieMemberResponse, MovieResponse,
    UpdateMovieRequest,
};
use super::service::MovieService;

// ============================================================================
// Query params
// ============================================================================

#[derive(Debug, Deserialize)]
struct FindMoviesQuery {
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_page_size")]
    page_size: i64,
    search: Option<String>,
    created_by: Option<String>,
}

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    20
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_movie(
    auth: AuthContext,
    svc: web::Data<MovieService>,
    body: web::Json<CreateMovieRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_MOVIES_WRITE)?;
    let movie = svc.create_movie(body.into_inner(), auth.user_id).await?;
    Ok(HttpResponse::Created().json(MovieResponse::from(movie)))
}

async fn find_movies(
    auth: AuthContext,
    svc: web::Data<MovieService>,
    query: web::Query<FindMoviesQuery>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_MOVIES_READ)?;
    let q = query.into_inner();
    let opts = PaginationOptions {
        page: q.page,
        page_size: q.page_size,
    };
    let filter = MovieFilter {
        search: q.search,
        created_by: q.created_by.map(UserId::from_string),
    };
    let paginated = svc.find_movies(&opts, &filter).await?;
    let items: Vec<MovieResponse> = paginated
        .items
        .into_iter()
        .map(MovieResponse::from)
        .collect();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "items": items,
        "pagination": paginated.pagination,
    })))
}

async fn get_movie(
    auth: AuthContext,
    svc: web::Data<MovieService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_MOVIES_READ)?;
    let movie = svc
        .get_movie(&MovieId::from_string(path.into_inner()))
        .await?;
    Ok(HttpResponse::Ok().json(MovieResponse::from(movie)))
}

async fn update_movie(
    auth: AuthContext,
    svc: web::Data<MovieService>,
    path: web::Path<String>,
    body: web::Json<UpdateMovieRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_MOVIES_WRITE)?;
    let movie = svc
        .update_movie(
            &MovieId::from_string(path.into_inner()),
            body.into_inner(),
            &auth.user_id,
        )
        .await?;
    Ok(HttpResponse::Ok().json(MovieResponse::from(movie)))
}

async fn delete_movie(
    auth: AuthContext,
    svc: web::Data<MovieService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_MOVIES_DELETE)?;
    svc.delete_movie(&MovieId::from_string(path.into_inner()), &auth.user_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn my_movies(
    auth: AuthContext,
    svc: web::Data<MovieService>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_MOVIES_READ)?;
    let movies = svc.get_user_movies(&auth.user_id).await?;
    let res: Vec<MovieResponse> = movies.into_iter().map(MovieResponse::from).collect();
    Ok(HttpResponse::Ok().json(res))
}

async fn add_member(
    auth: AuthContext,
    svc: web::Data<MovieService>,
    path: web::Path<String>,
    body: web::Json<AddMovieMemberRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_MOVIES_MEMBERS)?;
    let member = svc
        .add_member(
            &MovieId::from_string(path.into_inner()),
            body.into_inner(),
            &auth.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(MovieMemberResponse::from(&member)))
}

async fn remove_member(
    auth: AuthContext,
    svc: web::Data<MovieService>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_MOVIES_MEMBERS)?;
    let (movie_id, user_id) = path.into_inner();
    svc.remove_member(
        &MovieId::from_string(movie_id),
        &UserId::from_string(user_id),
        &auth.user_id,
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn list_members(
    auth: AuthContext,
    svc: web::Data<MovieService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_MOVIES_READ)?;
    let members = svc
        .list_members(&MovieId::from_string(path.into_inner()))
        .await?;
    let res: Vec<MovieMemberResponse> = members.iter().map(MovieMemberResponse::from).collect();
    Ok(HttpResponse::Ok().json(res))
}

async fn list_movie_scenes(
    auth: AuthContext,
    svc: web::Data<SceneService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_SCENES_READ)?;
    let scenes = svc
        .list_by_movie(&MovieId::from_string(path.into_inner()))
        .await?;
    let res: Vec<SceneResponse> = scenes.into_iter().map(SceneResponse::from).collect();
    Ok(HttpResponse::Ok().json(res))
}

// ============================================================================
// Route Configuration
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/movies")
            .route("", web::post().to(create_movie))
            .route("", web::get().to(find_movies))
            .route("/me", web::get().to(my_movies))
            .route("/{id}", web::get().to(get_movie))
            .route("/{id}", web::put().to(update_movie))
            .route("/{id}", web::delete().to(delete_movie))
            .route("/{id}/members", web::post().to(add_member))
            .route("/{id}/members", web::get().to(list_members))
            .route("/{id}/members/{user_id}", web::delete().to(remove_member))
            .route("/{id}/scenes", web::get().to(list_movie_scenes)),
    );
}
