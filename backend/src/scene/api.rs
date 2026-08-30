use actix_web::{HttpResponse, web};

use crate::error::AppError;
use crate::iam::auth::AuthContext;
use crate::iam::scopes;
use crate::kernel::SceneId;
use crate::track::{TrackResponse, TrackService};

use super::model::{CreateSceneRequest, SceneResponse, UpdateSceneRequest};
use super::service::SceneService;

// ============================================================================
// Handlers
// ============================================================================

async fn create_scene(
    auth: AuthContext,
    svc: web::Data<SceneService>,
    body: web::Json<CreateSceneRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_SCENES_WRITE)?;
    let scene = svc.create_scene(body.into_inner(), &auth.user_id).await?;
    Ok(HttpResponse::Created().json(SceneResponse::from(scene)))
}

async fn get_scene(
    auth: AuthContext,
    svc: web::Data<SceneService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_SCENES_READ)?;
    let scene = svc
        .get_scene(&SceneId::from_string(path.into_inner()))
        .await?;
    Ok(HttpResponse::Ok().json(SceneResponse::from(scene)))
}

async fn update_scene(
    auth: AuthContext,
    svc: web::Data<SceneService>,
    path: web::Path<String>,
    body: web::Json<UpdateSceneRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_SCENES_WRITE)?;
    let scene = svc
        .update_scene(
            &SceneId::from_string(path.into_inner()),
            body.into_inner(),
            &auth.user_id,
        )
        .await?;
    Ok(HttpResponse::Ok().json(SceneResponse::from(scene)))
}

async fn delete_scene(
    auth: AuthContext,
    svc: web::Data<SceneService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_SCENES_DELETE)?;
    svc.delete_scene(&SceneId::from_string(path.into_inner()), &auth.user_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn list_scene_tracks(
    auth: AuthContext,
    svc: web::Data<TrackService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_TRACKS_READ)?;
    let tracks = svc
        .list_by_scene(&SceneId::from_string(path.into_inner()))
        .await?;
    let res: Vec<TrackResponse> = tracks.into_iter().map(TrackResponse::from).collect();
    Ok(HttpResponse::Ok().json(res))
}

// ============================================================================
// Route Configuration
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/scenes")
            .route("", web::post().to(create_scene))
            .route("/{id}", web::get().to(get_scene))
            .route("/{id}", web::put().to(update_scene))
            .route("/{id}", web::delete().to(delete_scene))
            .route("/{id}/tracks", web::get().to(list_scene_tracks)),
    );
}
