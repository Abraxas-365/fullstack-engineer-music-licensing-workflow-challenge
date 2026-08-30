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

async fn create_track(
    auth: AuthContext,
    svc: web::Data<TrackService>,
    body: web::Json<CreateTrackRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_TRACKS_WRITE)?;
    let track = svc.create_track(body.into_inner(), auth.user_id).await?;
    Ok(HttpResponse::Created().json(TrackResponse::from(track)))
}

async fn get_track(
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

async fn update_track(
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

async fn delete_track(
    auth: AuthContext,
    svc: web::Data<TrackService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_TRACKS_DELETE)?;
    svc.delete_track(&TrackId::from_string(path.into_inner()), &auth.user_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn get_track_license(
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
