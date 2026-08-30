use actix_web::{HttpResponse, web};

use crate::error::AppError;
use crate::iam::auth::AuthContext;
use crate::iam::scopes;
use crate::kernel::{LabelId, UserId};
use crate::song::{SongResponse, SongService};

use super::model::{
    AddMemberRequest, CreateLabelRequest, LabelMemberResponse, LabelResponse, UpdateLabelRequest,
};
use super::service::LabelService;

// ============================================================================
// Handlers
// ============================================================================

async fn create_label(
    auth: AuthContext,
    svc: web::Data<LabelService>,
    body: web::Json<CreateLabelRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LABELS_WRITE)?;
    let label = svc.create_label(body.into_inner()).await?;
    Ok(HttpResponse::Created().json(LabelResponse::from(&label)))
}

async fn list_labels(
    auth: AuthContext,
    svc: web::Data<LabelService>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LABELS_READ)?;
    let labels = svc.list_labels().await?;
    let res: Vec<LabelResponse> = labels.iter().map(LabelResponse::from).collect();
    Ok(HttpResponse::Ok().json(res))
}

async fn get_label(
    auth: AuthContext,
    svc: web::Data<LabelService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LABELS_READ)?;
    let label = svc
        .get_label(&LabelId::from_string(path.into_inner()))
        .await?;
    Ok(HttpResponse::Ok().json(LabelResponse::from(&label)))
}

async fn update_label(
    auth: AuthContext,
    svc: web::Data<LabelService>,
    path: web::Path<String>,
    body: web::Json<UpdateLabelRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LABELS_WRITE)?;
    let label = svc
        .update_label(&LabelId::from_string(path.into_inner()), body.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(LabelResponse::from(&label)))
}

async fn delete_label(
    auth: AuthContext,
    svc: web::Data<LabelService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LABELS_DELETE)?;
    svc.delete_label(&LabelId::from_string(path.into_inner()))
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn add_member(
    auth: AuthContext,
    svc: web::Data<LabelService>,
    path: web::Path<String>,
    body: web::Json<AddMemberRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LABELS_MEMBERS)?;
    let member = svc
        .add_member(&LabelId::from_string(path.into_inner()), body.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(LabelMemberResponse::from(&member)))
}

async fn remove_member(
    auth: AuthContext,
    svc: web::Data<LabelService>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LABELS_MEMBERS)?;
    let (label_id, user_id) = path.into_inner();
    svc.remove_member(
        &LabelId::from_string(label_id),
        &UserId::from_string(user_id),
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn list_members(
    auth: AuthContext,
    svc: web::Data<LabelService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LABELS_READ)?;
    let members = svc
        .list_members(&LabelId::from_string(path.into_inner()))
        .await?;
    let res: Vec<LabelMemberResponse> = members.iter().map(LabelMemberResponse::from).collect();
    Ok(HttpResponse::Ok().json(res))
}

async fn get_user_labels(
    auth: AuthContext,
    svc: web::Data<LabelService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LABELS_READ)?;
    let labels = svc
        .get_user_labels(&UserId::from_string(path.into_inner()))
        .await?;
    let res: Vec<LabelResponse> = labels.iter().map(LabelResponse::from).collect();
    Ok(HttpResponse::Ok().json(res))
}

async fn list_label_songs(
    auth: AuthContext,
    svc: web::Data<SongService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_SONGS_READ)?;
    let songs = svc
        .list_by_label(&LabelId::from_string(path.into_inner()))
        .await?;
    let res: Vec<SongResponse> = songs.iter().map(SongResponse::from).collect();
    Ok(HttpResponse::Ok().json(res))
}

// ============================================================================
// Route Configuration
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/labels")
            .route("", web::post().to(create_label))
            .route("", web::get().to(list_labels))
            .route("/{id}", web::get().to(get_label))
            .route("/{id}", web::put().to(update_label))
            .route("/{id}", web::delete().to(delete_label))
            .route("/{id}/members", web::post().to(add_member))
            .route("/{id}/members", web::get().to(list_members))
            .route("/{id}/members/{user_id}", web::delete().to(remove_member))
            .route("/{id}/songs", web::get().to(list_label_songs)),
    );
    cfg.route("/users/{id}/labels", web::get().to(get_user_labels));
}
