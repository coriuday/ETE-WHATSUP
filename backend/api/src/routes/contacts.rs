use axum::{
    extract::{Multipart, Path, Query, State},
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
        contact::{
            BulkActionRequest, ContactListQuery, CreateContactRequest, CreateGroupRequest,
            UpdateContactRequest,
        },
        pagination::{ApiResponse, PaginatedResponse},
    },
    services::contact_service::ContactService,
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        // Contacts CRUD
        .route("/", get(list_contacts).post(create_contact))
        .route("/:id", get(get_contact).put(update_contact).delete(delete_contact))
        .route("/bulk", post(bulk_action))
        // Import
        .route("/import", post(import_contacts))
        .route("/import/:job_id", get(get_import_status))
        // Groups
        .route("/groups", get(list_groups).post(create_group))
        .route("/groups/:id", get(get_group).put(update_group).delete(delete_group))
        .route("/groups/:id/contacts", get(get_group_contacts))
        // Segments
        .route("/segments", get(list_segments).post(create_segment))
        .route("/segments/:id", delete(delete_segment))
        .with_state(state)
}

async fn list_contacts(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(query): Query<ContactListQuery>,
) -> AppResult<Json<ApiResponse<PaginatedResponse<serde_json::Value>>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let result = service.list_contacts(org_id, query).await?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn create_contact(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Json(req): Json<CreateContactRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let contact = service.create_contact(org_id, auth.id, req).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(contact))))
}

async fn get_contact(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let contact = service.get_contact(org_id, id).await?;
    Ok(Json(ApiResponse::ok(contact)))
}

async fn update_contact(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateContactRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let contact = service.update_contact(org_id, id, req).await?;
    Ok(Json(ApiResponse::ok(contact)))
}

async fn delete_contact(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    service.delete_contact(org_id, id).await?;
    Ok(Json(ApiResponse::with_message((), "Contact deleted")))
}

async fn bulk_action(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Json(req): Json<BulkActionRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let result = service.bulk_action(org_id, req).await?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn import_contacts(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    multipart: Multipart,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let job = service.start_import(org_id, auth.id, multipart).await?;
    Ok((StatusCode::ACCEPTED, Json(ApiResponse::ok(job))))
}

async fn get_import_status(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(job_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let status = service.get_import_status(org_id, job_id).await?;
    Ok(Json(ApiResponse::ok(status)))
}

async fn list_groups(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let groups = service.list_groups(org_id).await?;
    Ok(Json(ApiResponse::ok(groups)))
}

async fn create_group(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Json(req): Json<CreateGroupRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let group = service.create_group(org_id, auth.id, req).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(group))))
}

async fn get_group(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let group = service.get_group(org_id, id).await?;
    Ok(Json(ApiResponse::ok(group)))
}

async fn update_group(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateGroupRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let group = service.update_group(org_id, id, req).await?;
    Ok(Json(ApiResponse::ok(group)))
}

async fn delete_group(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    service.delete_group(org_id, id).await?;
    Ok(Json(ApiResponse::with_message((), "Group deleted")))
}

async fn get_group_contacts(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
    Query(query): Query<ContactListQuery>,
) -> AppResult<Json<ApiResponse<PaginatedResponse<serde_json::Value>>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let contacts = service.get_group_contacts(org_id, id, query).await?;
    Ok(Json(ApiResponse::ok(contacts)))
}

async fn list_segments(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let segments = service.list_segments(org_id).await?;
    Ok(Json(ApiResponse::ok(segments)))
}

async fn create_segment(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Json(body): Json<serde_json::Value>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    let segment = service.create_segment(org_id, auth.id, body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(segment))))
}

async fn delete_segment(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let service = ContactService::new(&state);
    service.delete_segment(org_id, id).await?;
    Ok(Json(ApiResponse::with_message((), "Segment deleted")))
}
