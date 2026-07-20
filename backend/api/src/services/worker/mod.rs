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

const RETRY_DELAYS_SECS: [i64; 3] = [60, 300, 900];

/// Spawn a campaign worker task (used by launch, resume, and crash recovery).
pub fn spawn_campaign_worker(state: AppState, campaign_id: Uuid) {
    tokio::spawn(async move {
        start_campaign_worker(state, campaign_id).await;
    });
}

/// On API startup: resume workers for campaigns still marked running.
pub async fn recover_running_campaigns(state: AppState) {
    let rows = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT id FROM campaigns
        WHERE status = 'running'
          AND deleted_at IS NULL
          AND EXISTS (
            SELECT 1 FROM message_queue_jobs j
            WHERE j.campaign_id = campaigns.id
              AND j.status IN ('pending', 'processing', 'retry')
          )
        "#,
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            tracing::info!("Recovering {} running campaign worker(s)", rows.len());
            for (id,) in rows {
                spawn_campaign_worker(state.clone(), id);
            }
        }
        Err(e) => tracing::error!("Failed to recover running campaigns: {:?}", e),
    }
}

/// Periodically reclaim jobs stuck in `processing` after a crash.
pub async fn run_job_reclaimer(state: AppState) {
    let mins = state.config.job_reclaim_minutes.max(1);
    tracing::info!("Job reclaimer started (timeout={}m)", mins);

    loop {
        if let Err(e) = reclaim_stuck_jobs(&state, mins).await {
            tracing::error!("Job reclaim error: {:?}", e);
        }
        sleep(Duration::from_secs(60)).await;
    }
}

async fn reclaim_stuck_jobs(state: &AppState, mins: u64) -> anyhow::Result<()> {
    let mins_i64 = mins as i64;
    let result = sqlx::query(
        r#"
        UPDATE message_queue_jobs
        SET status = 'retry',
            retry_at = NOW(),
            error = COALESCE(error, 'Reclaimed after stuck processing')
        WHERE status = 'processing'
          AND updated_at < NOW() - ($1 * INTERVAL '1 minute')
        "#,
    )
    .bind(mins_i64)
    .execute(&state.db)
    .await?;

    if result.rows_affected() > 0 {
        tracing::warn!("Reclaimed {} stuck processing job(s)", result.rows_affected());
    }
    Ok(())
}

pub async fn start_campaign_worker(state: AppState, campaign_id: Uuid) {
    let concurrency = state.config.wa_messages_per_second.max(1) as usize;
    tracing::info!(
        "Starting campaign worker: campaign={} concurrency={}",
        campaign_id,
        concurrency
    );

    let wa_creds = match load_wa_credentials(&state, campaign_id).await {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!(
                "Cannot start worker for campaign {}: failed to load WA credentials: {:?}",
                campaign_id,
                e
            );
            return;
        }
    };

    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

    loop {
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
                let wa_creds = wa_creds.clone();
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                tokio::spawn(async move {
                    let _permit = permit;
                    process_job(
                        &state_clone,
                        &wa_creds,
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
                let all_done = check_campaign_complete(&state, campaign_id).await;
                if all_done {
                    complete_campaign(&state, campaign_id).await;
                    break;
                }
                sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                tracing::error!("Failed to pull job for campaign {}: {:?}", campaign_id, e);
                sleep(Duration::from_secs(5)).await;
            }
        }

        sleep(Duration::from_millis(
            1000 / state.config.wa_messages_per_second.max(1),
        ))
        .await;
    }

    tracing::info!("Campaign worker finished for campaign {}", campaign_id);
}

struct WaCredentials {
    wa_account_id: Uuid,
    phone_number_id: String,
    access_token: String,
}

async fn load_wa_credentials(state: &AppState, campaign_id: Uuid) -> anyhow::Result<WaCredentials> {
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

    let access_token = match wa.access_token_enc {
        Some(enc) => {
            crate::utils::encryption::decrypt(&enc, &state.config.encryption_key).unwrap_or_default()
        }
        None => String::new(),
    };

    Ok(WaCredentials {
        wa_account_id: wa.id,
        phone_number_id: wa.phone_number_id.unwrap_or_default(),
        access_token,
    })
}

async fn process_job(
    state: &AppState,
    wa_creds: &WaCredentials,
    job_id: Uuid,
    campaign_id: Uuid,
    contact_id: Uuid,
    payload: serde_json::Value,
    attempts: i32,
    max_attempts: i32,
) {
    let result = dispatch_message(state, wa_creds, campaign_id, contact_id, &payload).await;

    match result {
        Ok(wa_message_id) => {
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
                let delay_secs = RETRY_DELAYS_SECS
                    .get(next_attempt as usize - 1)
                    .copied()
                    .unwrap_or(900);

                let retry_at = Utc::now() + chrono::Duration::seconds(delay_secs);

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

async fn dispatch_message(
    state: &AppState,
    wa_creds: &WaCredentials,
    campaign_id: Uuid,
    contact_id: Uuid,
    payload: &serde_json::Value,
) -> anyhow::Result<Option<String>> {
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

    let send_result = if let Some(template_name) = payload["template_name"].as_str() {
        let language = payload["language"].as_str().unwrap_or("en_US");
        let components = payload["components"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        wa_service
            .send_template(
                &wa_creds.phone_number_id,
                &wa_creds.access_token,
                &contact.phone_number,
                template_name,
                language,
                components,
            )
            .await
    } else {
        let body = payload["message_body"]
            .as_str()
            .unwrap_or("Hello from WhatsUp!");

        wa_service
            .send_text(
                &wa_creds.phone_number_id,
                &wa_creds.access_token,
                &contact.phone_number,
                body,
            )
            .await
    };

    let response = send_result?;
    let wa_message_id = response.messages.first().map(|m| m.id.clone());

    let _ = sqlx::query(
        r#"
        INSERT INTO messages (
            organization_id, wa_account_id, campaign_id, contact_id,
            wa_message_id, direction, type, body, status
        )
        SELECT $1, $2, $3, $4, $5, 'outbound', 'text', $6, 'sent'
        WHERE $5::text IS NULL
           OR NOT EXISTS (SELECT 1 FROM messages WHERE wa_message_id = $5)
        "#,
    )
    .bind(org_id)
    .bind(wa_creds.wa_account_id)
    .bind(campaign_id)
    .bind(contact_id)
    .bind(wa_message_id.as_deref())
    .bind(payload["message_body"].as_str().unwrap_or(""))
    .execute(&state.db)
    .await;

    Ok(wa_message_id)
}

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

#[cfg(test)]
mod tests {
    #[test]
    fn retry_delays_are_ordered() {
        assert!(super::RETRY_DELAYS_SECS[0] < super::RETRY_DELAYS_SECS[1]);
        assert!(super::RETRY_DELAYS_SECS[1] < super::RETRY_DELAYS_SECS[2]);
    }
}
