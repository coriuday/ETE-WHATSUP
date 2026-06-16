use serde_json::Value;
use uuid::Uuid;

use crate::AppState;

/// Fire-and-forget audit logger.
/// Writes to `audit_logs` without blocking the calling request.
pub struct AuditService<'a> {
    state: &'a AppState,
}

impl<'a> AuditService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    /// Log an action asynchronously (does not fail the caller on error).
    pub async fn log(
        &self,
        action: &str,
        user_id: Option<Uuid>,
        org_id: Option<Uuid>,
        resource_type: Option<&str>,
        resource_id: Option<Uuid>,
        metadata: Value,
    ) {
        let db = self.state.db.clone();
        let action = action.to_string();
        let resource_type = resource_type.map(|s| s.to_string());

        tokio::spawn(async move {
            let result = sqlx::query!(
                r#"
                INSERT INTO audit_logs
                    (org_id, user_id, action, resource_type, resource_id, metadata)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                org_id,
                user_id,
                action,
                resource_type,
                resource_id,
                metadata
            )
            .execute(&db)
            .await;

            if let Err(e) = result {
                tracing::warn!("Failed to write audit log (action={}): {:?}", action, e);
            }
        });
    }
}

// ── Convenience free function ────────────────────────────────────────────────

/// Shorthand for logging without constructing a service struct.
pub fn audit_log(
    state: &AppState,
    action: &str,
    user_id: Option<Uuid>,
    org_id: Option<Uuid>,
    resource_type: Option<&str>,
    resource_id: Option<Uuid>,
    metadata: Value,
) {
    let db = state.db.clone();
    let action = action.to_string();
    let resource_type = resource_type.map(|s| s.to_string());

    tokio::spawn(async move {
        let _ = sqlx::query!(
            r#"
            INSERT INTO audit_logs
                (org_id, user_id, action, resource_type, resource_id, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            org_id,
            user_id,
            action,
            resource_type,
            resource_id,
            metadata
        )
        .execute(&db)
        .await;
    });
}
