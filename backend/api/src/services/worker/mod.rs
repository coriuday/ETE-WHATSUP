use std::sync::Arc;

use chrono::Utc;
use serde_json::json;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::{
    services::{
        audit_service::audit_log,
        whatsapp_service::WhatsAppService,
    },
    AppState,
};

// ── Retry delays ─────────────────────────────────────────────────────────────

const RETRY_DELAYS_SECS: [i64; 3] = [60, 300, 900]; // 1 min, 5 min, 15 min

// ── Entry point ──────────────────────────────────────────────────────────────

/// Launch the campaign worker for a specific campaign.
/// Spawns a bounded pool of async tasks that pull jobs from
/// `message_queue_jobs` and dispatch WhatsApp messages.
pub async fn start_campaign_worker(state: AppState, campaign_id: Uuid) {
    let concurrency = state.config.wa_messages_per_second.max(1) as usize;
    tracing::info!(
        "Starting campaign worker: campaign={} concurrency={}",
        campaign_id,
        concurrency
    );

    // Use a semaphore to cap concurrency
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

    loop {
        // Check if campaign is still running
        let status = sqlx::query_scalar!(
            "SELECT status::text FROM campaigns WHERE id = $1",
            campaign_id
        )
        .fetch_optional(&state.db)
        .await;

        match status {
            Ok(Some(Some(s))) if s == "running" => {}
            _ => {
                tracing::info!("Campaign {} is no longer running — stopping worker", campaign_id);
                break;
            }
        }

        // Pull next pending job
        let job = sqlx::query!(
            r#"
            UPDATE message_queue_jobs
            SET status = 'processing'
            WHERE id = (
                SELECT id FROM message_queue_jobs
                WHERE campaign_id = $1
                  AND (status = 'pending' OR (status = 'retry' AND retry_at <= NOW()))
                ORDER BY scheduled_for ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, contact_id, payload, attempts, max_attempts
            "#,
            campaign_id
        )
        .fetch_optional(&state.db)
        .await;

        match job {
            Ok(Some(job)) => {
                let state_clone = state.clone();
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                tokio::spawn(async move {
                    let _permit = permit;
                    process_job(
                        &state_clone,
                        job.id,
                        campaign_id,
                        job.contact_id,
                        job.payload,
                        job.attempts,
                        job.max_attempts,
                    )
                    .await;
                });
            }
            Ok(None) => {
                // No pending jobs — check if campaign is complete
                let all_done = check_campaign_complete(&state, campaign_id).await;
                if all_done {
                    complete_campaign(&state, campaign_id).await;
                    break;
                }
                // Wait before polling again
                sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                tracing::error!("Failed to pull job for campaign {}: {:?}", campaign_id, e);
                sleep(Duration::from_secs(5)).await;
            }
        }

        // Throttle: respect wa_messages_per_second
        sleep(Duration::from_millis(
            1000 / state.config.wa_messages_per_second.max(1),
        ))
        .await;
    }

    tracing::info!("Campaign worker finished for campaign {}", campaign_id);
}

// ── Job processor ────────────────────────────────────────────────────────────

async fn process_job(
    state: &AppState,
    job_id: Uuid,
    campaign_id: Uuid,
    contact_id: Uuid,
    payload: serde_json::Value,
    attempts: i32,
    max_attempts: i32,
) {
    let result = dispatch_message(state, campaign_id, contact_id, &payload).await;

    match result {
        Ok(wa_message_id) => {
            // Mark job sent
            let _ = sqlx::query!(
                r#"
                UPDATE message_queue_jobs
                SET status = 'sent', processed_at = NOW(), attempts = attempts + 1
                WHERE id = $1
                "#,
                job_id
            )
            .execute(&state.db)
            .await;

            // Update campaign sent count
            let _ = sqlx::query!(
                "UPDATE campaigns SET sent_count = sent_count + 1 WHERE id = $1",
                campaign_id
            )
            .execute(&state.db)
            .await;

            tracing::debug!(
                "Job {} sent (wamid={})",
                job_id,
                wa_message_id.as_deref().unwrap_or("?")
            );
        }
        Err(e) => {
            let next_attempt = attempts + 1;

            if next_attempt >= max_attempts {
                // Permanently failed
                let _ = sqlx::query!(
                    r#"
                    UPDATE message_queue_jobs
                    SET status = 'failed', failed_at = NOW(), attempts = $1, error = $2
                    WHERE id = $3
                    "#,
                    next_attempt,
                    e.to_string(),
                    job_id
                )
                .execute(&state.db)
                .await;

                let _ = sqlx::query!(
                    "UPDATE campaigns SET failed_count = failed_count + 1 WHERE id = $1",
                    campaign_id
                )
                .execute(&state.db)
                .await;

                tracing::warn!("Job {} permanently failed: {}", job_id, e);
            } else {
                // Schedule retry with backoff
                let delay_secs = RETRY_DELAYS_SECS
                    .get(next_attempt as usize - 1)
                    .copied()
                    .unwrap_or(900);

                let retry_at = Utc::now()
                    + chrono::Duration::seconds(delay_secs);

                let _ = sqlx::query!(
                    r#"
                    UPDATE message_queue_jobs
                    SET status = 'retry', retry_at = $1, attempts = $2, error = $3
                    WHERE id = $4
                    "#,
                    retry_at,
                    next_attempt,
                    e.to_string(),
                    job_id
                )
                .execute(&state.db)
                .await;

                tracing::info!(
                    "Job {} queued for retry #{} in {}s",
                    job_id, next_attempt, delay_secs
                );
            }
        }
    }
}

// ── WhatsApp dispatch ────────────────────────────────────────────────────────

async fn dispatch_message(
    state: &AppState,
    campaign_id: Uuid,
    contact_id: Uuid,
    payload: &serde_json::Value,
) -> anyhow::Result<Option<String>> {
    // Load WA account credentials
    let wa = sqlx::query!(
        r#"
        SELECT wa.id, wa.phone_number_id, wa.access_token_enc
        FROM whatsapp_accounts wa
        JOIN campaigns c ON c.wa_account_id = wa.id
        WHERE c.id = $1
        "#,
        campaign_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| anyhow::anyhow!("No WhatsApp account linked to campaign"))?;

    // Load contact phone
    let contact = sqlx::query!(
        "SELECT phone_number, first_name FROM contacts WHERE id = $1",
        contact_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Contact not found"))?;

    let org_id: Uuid = payload["org_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_default();

    let wa_service = WhatsAppService::new(state);

    let access_token = match wa.access_token_enc {
        Some(enc) => {
            let key = &state.config.encryption_key;
            crate::utils::encryption::decrypt(&enc, key).unwrap_or_default()
        }
        None => String::new(),
    };

    let send_result = if let Some(template_name) = payload["template_name"].as_str() {
        // Template message
        let language = payload["language"].as_str().unwrap_or("en_US");
        let components = payload["components"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        wa_service
            .send_template(
                wa.phone_number_id.as_deref().unwrap_or_default(),
                &access_token,
                &contact.phone_number,
                template_name,
                language,
                components,
            )
            .await
    } else {
        // Text message
        let body = payload["message_body"]
            .as_str()
            .unwrap_or("Hello from WhatsUp!");

        wa_service
            .send_text(
                wa.phone_number_id.as_deref().unwrap_or_default(),
                &access_token,
                &contact.phone_number,
                body,
            )
            .await
    };

    let response = send_result?;
    let wa_message_id = response.messages.first().map(|m| m.id.clone());

    // Persist message record
    let _ = sqlx::query!(
        r#"
        INSERT INTO messages (
            organization_id, wa_account_id, campaign_id, contact_id,
            wa_message_id, direction, type, body, status
        )
        VALUES ($1, $2, $3, $4, $5, 'outbound', 'text', $6, 'sent')
        "#,
        org_id,
        wa.id,
        campaign_id,
        contact_id,
        wa_message_id.as_deref(),
        payload["message_body"].as_str().unwrap_or("")
    )
    .execute(&state.db)
    .await;

    Ok(wa_message_id)
}

// ── Completion check ─────────────────────────────────────────────────────────

async fn check_campaign_complete(state: &AppState, campaign_id: Uuid) -> bool {
    let pending = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM message_queue_jobs
        WHERE campaign_id = $1 AND status IN ('pending', 'processing', 'retry')
        "#,
        campaign_id
    )
    .fetch_one(&state.db)
    .await;

    matches!(pending, Ok(Some(0)) | Ok(None))
}

async fn complete_campaign(state: &AppState, campaign_id: Uuid) {
    let result = sqlx::query!(
        r#"
        UPDATE campaigns
        SET status = 'completed', completed_at = NOW()
        WHERE id = $1 AND status = 'running'
        "#,
        campaign_id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            tracing::info!("Campaign {} marked as completed", campaign_id);
            audit_log(
                state,
                "campaign.completed",
                None,
                None,
                Some("campaign"),
                Some(campaign_id),
                json!({ "campaign_id": campaign_id }),
            );
        }
        Err(e) => {
            tracing::error!("Failed to mark campaign {} as completed: {:?}", campaign_id, e);
        }
    }
}
