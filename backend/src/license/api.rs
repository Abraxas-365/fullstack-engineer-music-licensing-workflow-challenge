use actix_web::{HttpResponse, web};
use serde::Deserialize;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::error::AppError;
use crate::iam::auth::AuthContext;
use crate::iam::scopes;
use crate::kernel::LicenseRequestId;

use super::model::{
    CreateLicenseRequest, LicenseEvent, LicenseOfferResponse, LicenseRequestResponse, OfferTerms,
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

/// SSE endpoint — streams license negotiation events to the connected client.
///
/// The client receives lightweight JSON events; it should call the REST API
/// to fetch full details when needed.
async fn events(_auth: AuthContext, svc: web::Data<LicenseService>) -> HttpResponse {
    let rx: broadcast::Receiver<LicenseEvent> = svc.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => {
            let json = serde_json::to_string(&event).ok()?;
            Some(Ok::<_, actix_web::Error>(web::Bytes::from(format!(
                "data: {json}\n\n"
            ))))
        }
        Err(_) => None,
    });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}

// ============================================================================
// Route Configuration
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/licenses")
            .route("/events", web::get().to(events))
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
