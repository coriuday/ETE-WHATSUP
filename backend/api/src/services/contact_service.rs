use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::contact::{
        BulkActionRequest, BulkContactAction, ContactListQuery, CreateContactRequest,
        CreateGroupRequest, UpdateContactRequest,
    },
    utils::phone::normalize_phone,
    AppState,
};

pub struct ContactService<'a> {
    state: &'a AppState,
}

impl<'a> ContactService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list_contacts(
        &self,
        org_id: Uuid,
        query: ContactListQuery,
    ) -> AppResult<crate::models::pagination::PaginatedResponse<serde_json::Value>> {
        use crate::models::pagination::{PaginatedResponse, PaginationQuery};
        let pq = PaginationQuery { page: query.page, limit: query.limit };

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM contacts WHERE organization_id = $1 AND deleted_at IS NULL",
            org_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .unwrap_or(0);

        let contacts = sqlx::query!(
            r#"
            SELECT id, phone_number, first_name, last_name, email,
                   wa_status::text, wa_opted_in, tags, source::text,
                   last_contacted_at, last_replied_at, created_at
            FROM contacts
            WHERE organization_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            org_id,
            pq.limit_i64(),
            pq.offset()
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        let data = contacts
            .into_iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "phone_number": c.phone_number,
                    "first_name": c.first_name,
                    "last_name": c.last_name,
                    "email": c.email,
                    "wa_status": c.wa_status,
                    "wa_opted_in": c.wa_opted_in,
                    "tags": c.tags,
                    "source": c.source,
                    "last_contacted_at": c.last_contacted_at,
                    "last_replied_at": c.last_replied_at,
                    "created_at": c.created_at,
                })
            })
            .collect();

        Ok(PaginatedResponse::new(data, total, &pq))
    }

    pub async fn create_contact(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        req: CreateContactRequest,
    ) -> AppResult<serde_json::Value> {
        let phone = normalize_phone(&req.phone_number)
            .ok_or_else(|| AppError::Validation("Invalid phone number format".into()))?;

        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO contacts (organization_id, phone_number, first_name, last_name, email, tags, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (organization_id, phone_number) DO UPDATE
                SET first_name = COALESCE(EXCLUDED.first_name, contacts.first_name),
                    last_name = COALESCE(EXCLUDED.last_name, contacts.last_name),
                    email = COALESCE(EXCLUDED.email, contacts.email)
            RETURNING id
            "#,
            org_id,
            phone,
            req.first_name,
            req.last_name,
            req.email,
            &req.tags.unwrap_or_default(),
            user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        // Add to groups if specified
        if let Some(group_ids) = req.group_ids {
            for group_id in group_ids {
                let _ = sqlx::query!(
                    "INSERT INTO contact_group_members (contact_id, group_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    id, group_id
                )
                .execute(&self.state.db)
                .await;
            }
        }

        // Fire n8n event
        let n8n = crate::services::n8n_service::N8nService::new(self.state);
        n8n.fire_event(org_id, crate::services::n8n_service::N8nEvent::ContactCreated {
            contact_id: id,
            org_id,
        }).await;

        self.get_contact(org_id, id).await
    }

    pub async fn get_contact(&self, org_id: Uuid, id: Uuid) -> AppResult<serde_json::Value> {
        let c = sqlx::query!(
            r#"
            SELECT id, phone_number, first_name, last_name, email, avatar_url,
                   wa_status::text, wa_opted_in, tags, source::text, custom_fields,
                   total_msgs_sent, total_msgs_received, last_contacted_at, last_replied_at,
                   created_at, updated_at
            FROM contacts
            WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL
            "#,
            id, org_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Contact".into()))?;

        Ok(serde_json::json!({
            "id": c.id,
            "phone_number": c.phone_number,
            "first_name": c.first_name,
            "last_name": c.last_name,
            "email": c.email,
            "avatar_url": c.avatar_url,
            "wa_status": c.wa_status,
            "wa_opted_in": c.wa_opted_in,
            "tags": c.tags,
            "source": c.source,
            "custom_fields": c.custom_fields,
            "stats": {
                "msgs_sent": c.total_msgs_sent,
                "msgs_received": c.total_msgs_received,
            },
            "last_contacted_at": c.last_contacted_at,
            "last_replied_at": c.last_replied_at,
            "created_at": c.created_at,
            "updated_at": c.updated_at,
        }))
    }

    pub async fn update_contact(
        &self,
        org_id: Uuid,
        id: Uuid,
        req: UpdateContactRequest,
    ) -> AppResult<serde_json::Value> {
        sqlx::query!(
            r#"
            UPDATE contacts
            SET first_name = COALESCE($1, first_name),
                last_name = COALESCE($2, last_name),
                email = COALESCE($3, email),
                tags = COALESCE($4, tags)
            WHERE id = $5 AND organization_id = $6 AND deleted_at IS NULL
            "#,
            req.first_name, req.last_name, req.email,
            req.tags.as_deref(),
            id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        self.get_contact(org_id, id).await
    }

    pub async fn delete_contact(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        sqlx::query!(
            "UPDATE contacts SET deleted_at = NOW() WHERE id = $1 AND organization_id = $2",
            id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;
        Ok(())
    }

    pub async fn bulk_action(
        &self,
        org_id: Uuid,
        req: BulkActionRequest,
    ) -> AppResult<serde_json::Value> {
        let count = req.contact_ids.len();

        match req.action {
            BulkContactAction::Delete => {
                sqlx::query!(
                    "UPDATE contacts SET deleted_at = NOW() WHERE id = ANY($1) AND organization_id = $2",
                    &req.contact_ids,
                    org_id
                )
                .execute(&self.state.db)
                .await
                .map_err(AppError::Database)?;
            }
            BulkContactAction::AddToGroup => {
                if let Some(group_id) = req.group_id {
                    for contact_id in &req.contact_ids {
                        let _ = sqlx::query!(
                            "INSERT INTO contact_group_members (contact_id, group_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                            contact_id, group_id
                        )
                        .execute(&self.state.db)
                        .await;
                    }
                }
            }
            BulkContactAction::Unsubscribe => {
                sqlx::query!(
                    "UPDATE contacts SET wa_status = 'unsubscribed', opted_out_at = NOW() WHERE id = ANY($1) AND organization_id = $2",
                    &req.contact_ids, org_id
                )
                .execute(&self.state.db)
                .await
                .map_err(AppError::Database)?;
            }
            _ => {}
        }

        Ok(serde_json::json!({ "affected": count }))
    }

    pub async fn start_import(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        mut multipart: axum::extract::Multipart,
    ) -> AppResult<serde_json::Value> {
        // Extract the file from multipart
        while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
            let filename = field.file_name().unwrap_or("import.csv").to_string();
            let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;

            // Store in S3
            let storage = crate::services::storage_service::StorageService::new(self.state);
            let file_url = storage
                .upload_bytes(&format!("imports/{}/{}", org_id, filename), &data, "text/csv")
                .await
                .map_err(|e| AppError::Storage(e.to_string()))?;

            // Create import job
            let job_id = sqlx::query_scalar!(
                r#"
                INSERT INTO contact_imports (organization_id, file_name, file_url, status, created_by)
                VALUES ($1, $2, $3, 'pending', $4)
                RETURNING id
                "#,
                org_id, filename, file_url, user_id
            )
            .fetch_one(&self.state.db)
            .await
            .map_err(AppError::Database)?;

            // TODO: Spawn background task to process CSV
            // For now, return job ID immediately
            return Ok(serde_json::json!({
                "job_id": job_id,
                "status": "pending",
                "file_name": filename,
                "message": "Import job created. Processing will begin shortly."
            }));
        }

        Err(AppError::BadRequest("No file found in request".into()))
    }

    pub async fn get_import_status(&self, org_id: Uuid, job_id: Uuid) -> AppResult<serde_json::Value> {
        let job = sqlx::query!(
            r#"
            SELECT id, file_name, status::text, total_rows, processed_rows,
                   imported_count, skipped_count, error_count, created_at, completed_at
            FROM contact_imports
            WHERE id = $1 AND organization_id = $2
            "#,
            job_id, org_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Import job".into()))?;

        Ok(serde_json::json!({
            "id": job.id,
            "file_name": job.file_name,
            "status": job.status,
            "total_rows": job.total_rows,
            "processed_rows": job.processed_rows,
            "imported": job.imported_count,
            "skipped": job.skipped_count,
            "errors": job.error_count,
            "created_at": job.created_at,
            "completed_at": job.completed_at,
        }))
    }

    pub async fn list_groups(&self, org_id: Uuid) -> AppResult<serde_json::Value> {
        let groups = sqlx::query!(
            "SELECT id, name, description, color, contact_count, created_at FROM contact_groups WHERE organization_id = $1 ORDER BY name",
            org_id
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        let data: Vec<serde_json::Value> = groups.iter().map(|g| serde_json::json!({
            "id": g.id, "name": g.name, "description": g.description,
            "color": g.color, "contact_count": g.contact_count, "created_at": g.created_at
        })).collect();

        Ok(serde_json::json!({ "groups": data }))
    }

    pub async fn create_group(&self, org_id: Uuid, user_id: Uuid, req: CreateGroupRequest) -> AppResult<serde_json::Value> {
        let id = sqlx::query_scalar!(
            "INSERT INTO contact_groups (organization_id, name, description, color, created_by) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            org_id, req.name, req.description, req.color, user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        self.get_group(org_id, id).await
    }

    pub async fn get_group(&self, org_id: Uuid, id: Uuid) -> AppResult<serde_json::Value> {
        let g = sqlx::query!(
            "SELECT id, name, description, color, contact_count, created_at FROM contact_groups WHERE id = $1 AND organization_id = $2",
            id, org_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Group".into()))?;

        Ok(serde_json::json!({
            "id": g.id, "name": g.name, "description": g.description,
            "color": g.color, "contact_count": g.contact_count, "created_at": g.created_at
        }))
    }

    pub async fn update_group(&self, org_id: Uuid, id: Uuid, req: CreateGroupRequest) -> AppResult<serde_json::Value> {
        sqlx::query!(
            "UPDATE contact_groups SET name = $1, description = $2, color = $3 WHERE id = $4 AND organization_id = $5",
            req.name, req.description, req.color, id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;
        self.get_group(org_id, id).await
    }

    pub async fn delete_group(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        sqlx::query!("DELETE FROM contact_groups WHERE id = $1 AND organization_id = $2", id, org_id)
            .execute(&self.state.db)
            .await
            .map_err(AppError::Database)?;
        Ok(())
    }

    pub async fn get_group_contacts(
        &self,
        org_id: Uuid,
        group_id: Uuid,
        query: ContactListQuery,
    ) -> AppResult<crate::models::pagination::PaginatedResponse<serde_json::Value>> {
        use crate::models::pagination::{PaginatedResponse, PaginationQuery};
        let pq = PaginationQuery { page: query.page, limit: query.limit };

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM contact_group_members cgm JOIN contacts c ON c.id = cgm.contact_id WHERE cgm.group_id = $1 AND c.organization_id = $2",
            group_id, org_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .unwrap_or(0);

        let contacts = sqlx::query!(
            r#"
            SELECT c.id, c.phone_number, c.first_name, c.last_name, c.wa_status::text, c.tags
            FROM contacts c
            JOIN contact_group_members cgm ON cgm.contact_id = c.id
            WHERE cgm.group_id = $1 AND c.organization_id = $2 AND c.deleted_at IS NULL
            LIMIT $3 OFFSET $4
            "#,
            group_id, org_id, pq.limit_i64(), pq.offset()
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        let data = contacts.into_iter().map(|c| serde_json::json!({
            "id": c.id, "phone_number": c.phone_number,
            "first_name": c.first_name, "last_name": c.last_name,
            "wa_status": c.wa_status, "tags": c.tags,
        })).collect();

        Ok(PaginatedResponse::new(data, total, &pq))
    }

    pub async fn list_segments(&self, org_id: Uuid) -> AppResult<serde_json::Value> {
        let segs = sqlx::query!(
            "SELECT id, name, description, filter_rules, contact_count, created_at FROM contact_segments WHERE organization_id = $1",
            org_id
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        let data: Vec<serde_json::Value> = segs.iter().map(|s| serde_json::json!({
            "id": s.id, "name": s.name, "description": s.description,
            "filter_rules": s.filter_rules, "contact_count": s.contact_count,
        })).collect();

        Ok(serde_json::json!({ "segments": data }))
    }

    pub async fn create_segment(&self, org_id: Uuid, user_id: Uuid, body: serde_json::Value) -> AppResult<serde_json::Value> {
        let name = body["name"].as_str().unwrap_or("Segment").to_string();
        let filter_rules = body.get("filter_rules").cloned().unwrap_or_default();

        let id = sqlx::query_scalar!(
            "INSERT INTO contact_segments (organization_id, name, filter_rules, created_by) VALUES ($1, $2, $3, $4) RETURNING id",
            org_id, name, filter_rules, user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        Ok(serde_json::json!({ "id": id, "name": name }))
    }

    pub async fn delete_segment(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        sqlx::query!("DELETE FROM contact_segments WHERE id = $1 AND organization_id = $2", id, org_id)
            .execute(&self.state.db)
            .await
            .map_err(AppError::Database)?;
        Ok(())
    }
}
