use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::{RequireOrgAdmin, RequireOrgViewer},
    models::{
        campaign::{
            CampaignListQuery, CreateCampaignRequest, ScheduleCampaignRequest,
            UpdateCampaignRequest,
        },
        pagination::ApiResponse,
    },
    services::campaign_service::CampaignService,
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_campaigns).post(create_campaign))
        .route("/:id", get(get_campaign).put(update_campaign).delete(delete_campaign))
        .route("/:id/launch", post(launch_campaign))
        .route("/:id/schedule", post(schedule_campaign))
        .route("/:id/pause", post(pause_campaign))
        .route("/:id/resume", post(resume_campaign))
        .route("/:id/cancel", post(cancel_campaign))
        .route("/:id/clone", post(clone_campaign))
        .route("/:id/stats", get(get_campaign_stats))
        .route("/:id/messages", get(get_campaign_messages))
        .with_state(state)
}

async fn list_campaigns(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(query): Query<CampaignListQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let result = service.list_campaigns(org_id, query).await?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn create_campaign(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Json(req): Json<CreateCampaignRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let campaign = service.create_campaign(org_id, auth.id, req).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(campaign))))
}

async fn get_campaign(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let campaign = service.get_campaign(org_id, id).await?;
    Ok(Json(ApiResponse::ok(campaign)))
}

async fn update_campaign(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCampaignRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let campaign = service.update_campaign(org_id, id, req).await?;
    Ok(Json(ApiResponse::ok(campaign)))
}

async fn delete_campaign(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    service.delete_campaign(org_id, id).await?;
    Ok(Json(ApiResponse::with_message((), "Campaign deleted")))
}

async fn launch_campaign(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let result = service.launch_campaign(org_id, id, auth.id).await?;
    Ok(Json(ApiResponse::with_message(result, "Campaign launched successfully")))
}

async fn schedule_campaign(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
    Json(req): Json<ScheduleCampaignRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let result = service.schedule_campaign(org_id, id, req).await?;
    Ok(Json(ApiResponse::with_message(result, "Campaign scheduled successfully")))
}

async fn pause_campaign(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let result = service.pause_campaign(org_id, id).await?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn resume_campaign(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let result = service.resume_campaign(org_id, id).await?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn cancel_campaign(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    service.cancel_campaign(org_id, id).await?;
    Ok(Json(ApiResponse::with_message((), "Campaign cancelled")))
}

async fn clone_campaign(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let clone = service.clone_campaign(org_id, id, auth.id).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(clone))))
}

async fn get_campaign_stats(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let stats = service.get_campaign_stats(org_id, id).await?;
    Ok(Json(ApiResponse::ok(stats)))
}

async fn get_campaign_messages(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = CampaignService::new(&state);
    let messages = service.get_campaign_messages(org_id, id, query).await?;
    Ok(Json(ApiResponse::ok(messages)))
}
