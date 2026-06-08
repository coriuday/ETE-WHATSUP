use axum::{extract::{Path, State}, http::StatusCode, routing::{delete, get, post, put}, Json, Router};
use uuid::Uuid;
use crate::{errors::{AppError, AppResult}, middleware::auth::AuthUser, models::pagination::ApiResponse, AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_orgs).post(create_org))
        .route("/:id", get(get_org).put(update_org))
        .route("/:id/members", get(list_members).post(invite_member))
        .route("/:id/members/:user_id", delete(remove_member))
        .route("/:id/usage", get(get_usage))
        .with_state(state)
}

async fn list_orgs(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let orgs = sqlx::query!(
        r#"SELECT o.id, o.name, o.slug, o.plan::text, o.status::text, o.logo_url, om.role::text
           FROM organizations o JOIN org_members om ON om.organization_id = o.id
           WHERE om.user_id = $1 AND o.deleted_at IS NULL"#,
        auth.id
    ).fetch_all(&state.db).await.map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = orgs.iter().map(|o| serde_json::json!({
        "id": o.id, "name": o.name, "slug": o.slug, "plan": o.plan,
        "status": o.status, "logo_url": o.logo_url, "role": o.role
    })).collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "organizations": data }))))
}

async fn create_org(State(state): State<AppState>, auth: AuthUser, Json(req): Json<crate::models::organization::CreateOrganizationRequest>) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    use validator::Validate;
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let slug = req.name.to_lowercase().replace(' ', "-");
    let id = sqlx::query_scalar!(
        "INSERT INTO organizations (name, slug, owner_id, website, industry, country, timezone) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        req.name, slug, auth.id, req.website, req.industry, req.country,
        req.timezone.as_deref().unwrap_or("Asia/Kolkata")
    ).fetch_one(&state.db).await.map_err(AppError::Database)?;

    sqlx::query!("INSERT INTO org_members (organization_id, user_id, role) VALUES ($1, $2, 'owner')", id, auth.id)
        .execute(&state.db).await.map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(ApiResponse::ok(serde_json::json!({ "id": id })))))
}

async fn get_org(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let o = sqlx::query!(
        r#"SELECT id, name, slug, logo_url, website, industry, country, timezone, plan::text, status::text,
                  max_contacts, max_campaigns, monthly_msg_quota, msgs_sent_this_month, created_at
           FROM organizations WHERE id = $1 AND deleted_at IS NULL"#,
        id
    ).fetch_optional(&state.db).await.map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Organization".into()))?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "id": o.id, "name": o.name, "slug": o.slug, "logo_url": o.logo_url,
        "website": o.website, "industry": o.industry, "country": o.country,
        "timezone": o.timezone, "plan": o.plan, "status": o.status,
        "limits": { "contacts": o.max_contacts, "campaigns": o.max_campaigns, "monthly_msgs": o.monthly_msg_quota },
        "usage": { "msgs_this_month": o.msgs_sent_this_month },
    }))))
}

async fn update_org(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(req): Json<crate::models::organization::UpdateOrganizationRequest>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    sqlx::query!(
        "UPDATE organizations SET name = COALESCE($1, name), logo_url = COALESCE($2, logo_url), website = COALESCE($3, website) WHERE id = $4",
        req.name, req.logo_url, req.website, id
    ).execute(&state.db).await.map_err(AppError::Database)?;
    get_org(State(state), auth, Path(id)).await
}

async fn list_members(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let members = sqlx::query!(
        r#"SELECT u.id, u.email, u.first_name, u.last_name, u.avatar_url, om.role::text, om.joined_at
           FROM users u JOIN org_members om ON om.user_id = u.id WHERE om.organization_id = $1"#,
        id
    ).fetch_all(&state.db).await.map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = members.iter().map(|m| serde_json::json!({
        "id": m.id, "email": m.email, "first_name": m.first_name, "last_name": m.last_name,
        "avatar_url": m.avatar_url, "role": m.role, "joined_at": m.joined_at
    })).collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "members": data }))))
}

async fn invite_member(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(req): Json<crate::models::organization::InviteMemberRequest>) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    use validator::Validate;
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let token = crate::utils::jwt::generate_secure_token(32);
    let invite_id = sqlx::query_scalar!(
        "INSERT INTO org_invitations (organization_id, invited_by, email, role, token) VALUES ($1, $2, $3, $4::member_role, $5) RETURNING id",
        id, auth.id, req.email, req.role as _, token
    ).fetch_one(&state.db).await.map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(ApiResponse::with_message(serde_json::json!({ "invitation_id": invite_id }), "Invitation sent"))))
}

async fn remove_member(State(state): State<AppState>, auth: AuthUser, Path((org_id, user_id)): Path<(Uuid, Uuid)>) -> AppResult<Json<ApiResponse<()>>> {
    sqlx::query!("DELETE FROM org_members WHERE organization_id = $1 AND user_id = $2", org_id, user_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message((), "Member removed")))
}

async fn get_usage(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let o = sqlx::query!(
        "SELECT max_contacts, max_campaigns, monthly_msg_quota, msgs_sent_this_month FROM organizations WHERE id = $1",
        id
    ).fetch_one(&state.db).await.map_err(AppError::Database)?;

    let contact_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM contacts WHERE organization_id = $1 AND deleted_at IS NULL", id)
        .fetch_one(&state.db).await.map_err(AppError::Database)?.unwrap_or(0);

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "contacts": { "used": contact_count, "limit": o.max_contacts },
        "messages": { "used_this_month": o.msgs_sent_this_month, "limit": o.monthly_msg_quota },
    }))))
}
