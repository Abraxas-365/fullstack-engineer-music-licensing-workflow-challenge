use actix_web::{HttpResponse, web};

use crate::error::AppError;
use crate::iam::auth::AuthContext;
use crate::iam::scopes;
use crate::kernel::TrackId;
use crate::license::{LicenseRequestResponse, LicenseService};

use super::model::{CreateTrackRequest, TrackResponse, UpdateTrackRequest};
use super::service::TrackService;

// ============================================================================
// Handlers
// ============================================================================

/// Create a track (place a song into a scene)
#[utoipa::path(
    post,
    path = "/api/tracks",
    tag = "Tracks",
    security(("bearer_auth" = [])),
    request_body = CreateTrackRequest,
    responses(
        (status = 201, description = "Track created", body = TrackResponse),
        (status = 400, description = "Validation error", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Scene or song not found", body = crate::error::ErrorResponse),
    )
)]
pub async fn create_track(
    auth: AuthContext,
    svc: web::Data<TrackService>,
    body: web::Json<CreateTrackRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_TRACKS_WRITE)?;
    let track = svc.create_track(body.into_inner(), auth.user_id).await?;
    Ok(HttpResponse::Created().json(TrackResponse::from(track)))
}

/// Get a track
#[utoipa::path(
    get,
    path = "/api/tracks/{id}",
    tag = "Tracks",
    security(("bearer_auth" = [])),
    params(("id" = String, Path, description = "Track id")),
    responses(
        (status = 200, description = "Track", body = TrackResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Track not found", body = crate::error::ErrorResponse),
    )
)]
pub async fn get_track(
    auth: AuthContext,
    svc: web::Data<TrackService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_TRACKS_READ)?;
    let track = svc
        .get_track(&TrackId::from_string(path.into_inner()))
        .await?;
    Ok(HttpResponse::Ok().json(TrackResponse::from(track)))
}

/// Update a track
#[utoipa::path(
    put,
    path = "/api/tracks/{id}",
    tag = "Tracks",
    security(("bearer_auth" = [])),
    params(("id" = String, Path, description = "Track id")),
    request_body = UpdateTrackRequest,
    responses(
        (status = 200, description = "Track updated", body = TrackResponse),
        (status = 400, description = "Validation error", body = crate::error::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Track not found", body = crate::error::ErrorResponse),
    )
)]
pub async fn update_track(
    auth: AuthContext,
    svc: web::Data<TrackService>,
    path: web::Path<String>,
    body: web::Json<UpdateTrackRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_TRACKS_WRITE)?;
    let track = svc
        .update_track(
            &TrackId::from_string(path.into_inner()),
            body.into_inner(),
            &auth.user_id,
        )
        .await?;
    Ok(HttpResponse::Ok().json(TrackResponse::from(track)))
}

/// Delete a track
#[utoipa::path(
    delete,
    path = "/api/tracks/{id}",
    tag = "Tracks",
    security(("bearer_auth" = [])),
    params(("id" = String, Path, description = "Track id")),
    responses(
        (status = 204, description = "Track deleted"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Track not found", body = crate::error::ErrorResponse),
    )
)]
pub async fn delete_track(
    auth: AuthContext,
    svc: web::Data<TrackService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_TRACKS_DELETE)?;
    svc.delete_track(&TrackId::from_string(path.into_inner()), &auth.user_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Get the license request associated with a track, if any
#[utoipa::path(
    get,
    path = "/api/tracks/{id}/license",
    tag = "Tracks",
    security(("bearer_auth" = [])),
    params(("id" = String, Path, description = "Track id")),
    responses(
        (status = 200, description = "License request, or null if none exists", body = Option<LicenseRequestResponse>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Track not found", body = crate::error::ErrorResponse),
    )
)]
pub async fn get_track_license(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_READ)?;
    let license = svc
        .get_by_track(&TrackId::from_string(path.into_inner()))
        .await?;
    match license {
        Some(l) => Ok(HttpResponse::Ok().json(LicenseRequestResponse::from(l))),
        None => Ok(HttpResponse::Ok().json(serde_json::Value::Null)),
    }
}

// ============================================================================
// Route Configuration
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tracks")
            .route("", web::post().to(create_track))
            .route("/{id}", web::get().to(get_track))
            .route("/{id}", web::put().to(update_track))
            .route("/{id}", web::delete().to(delete_track))
            .route("/{id}/license", web::get().to(get_track_license)),
    );
}
