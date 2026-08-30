use actix_web::{HttpResponse, web};
use serde::Deserialize;

use crate::error::AppError;
use crate::iam::auth::AuthContext;
use crate::iam::scopes;
use crate::kernel::LicenseRequestId;

use super::model::{
    CreateLicenseRequest, LicenseOfferResponse, LicenseRequestResponse, OfferTerms,
};
use super::service::LicenseService;

// ============================================================================
// Request bodies
// ============================================================================

#[derive(Debug, Deserialize)]
struct RejectBody {
    reason: String,
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_license(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    body: web::Json<CreateLicenseRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_WRITE)?;
    let (license, offer) = svc.create_license(body.into_inner(), auth.user_id).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "license": LicenseRequestResponse::from(license),
        "offer": LicenseOfferResponse::from(offer),
    })))
}

async fn get_license(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_READ)?;
    let license = svc
        .get_license(&LicenseRequestId::from_string(path.into_inner()))
        .await?;
    Ok(HttpResponse::Ok().json(LicenseRequestResponse::from(license)))
}

async fn list_offers(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_READ)?;
    let offers = svc
        .list_offers(&LicenseRequestId::from_string(path.into_inner()))
        .await?;
    let res: Vec<LicenseOfferResponse> =
        offers.into_iter().map(LicenseOfferResponse::from).collect();
    Ok(HttpResponse::Ok().json(res))
}

async fn revise_draft(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
    body: web::Json<OfferTerms>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_WRITE)?;
    let offer = svc
        .revise_draft(
            &LicenseRequestId::from_string(path.into_inner()),
            body.into_inner(),
            auth.user_id,
        )
        .await?;
    Ok(HttpResponse::Ok().json(LicenseOfferResponse::from(offer)))
}

async fn submit(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_WRITE)?;
    let license = svc
        .submit(
            &LicenseRequestId::from_string(path.into_inner()),
            auth.user_id,
        )
        .await?;
    Ok(HttpResponse::Ok().json(LicenseRequestResponse::from(license)))
}

async fn counter_offer(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
    body: web::Json<OfferTerms>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_NEGOTIATE)?;
    let offer = svc
        .counter_offer(
            &LicenseRequestId::from_string(path.into_inner()),
            body.into_inner(),
            auth.user_id,
        )
        .await?;
    Ok(HttpResponse::Ok().json(LicenseOfferResponse::from(offer)))
}

async fn accept(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_NEGOTIATE)?;
    let license = svc
        .accept(
            &LicenseRequestId::from_string(path.into_inner()),
            auth.user_id,
        )
        .await?;
    Ok(HttpResponse::Ok().json(LicenseRequestResponse::from(license)))
}

async fn reject(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
    body: web::Json<RejectBody>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_NEGOTIATE)?;
    let license = svc
        .reject(
            &LicenseRequestId::from_string(path.into_inner()),
            auth.user_id,
            body.into_inner().reason,
        )
        .await?;
    Ok(HttpResponse::Ok().json(LicenseRequestResponse::from(license)))
}

async fn cancel(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_WRITE)?;
    let license = svc
        .cancel(
            &LicenseRequestId::from_string(path.into_inner()),
            auth.user_id,
        )
        .await?;
    Ok(HttpResponse::Ok().json(LicenseRequestResponse::from(license)))
}

async fn delete_license(
    auth: AuthContext,
    svc: web::Data<LicenseService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    auth.require_scope(scopes::SCOPE_LICENSES_DELETE)?;
    svc.delete_license(
        &LicenseRequestId::from_string(path.into_inner()),
        auth.user_id,
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

// ============================================================================
// Route Configuration
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/licenses")
            .route("", web::post().to(create_license))
            .route("/{id}", web::get().to(get_license))
            .route("/{id}", web::delete().to(delete_license))
            .route("/{id}/offers", web::get().to(list_offers))
            .route("/{id}/revise", web::post().to(revise_draft))
            .route("/{id}/submit", web::post().to(submit))
            .route("/{id}/counter", web::post().to(counter_offer))
            .route("/{id}/accept", web::post().to(accept))
            .route("/{id}/reject", web::post().to(reject))
            .route("/{id}/cancel", web::post().to(cancel)),
    );
}
