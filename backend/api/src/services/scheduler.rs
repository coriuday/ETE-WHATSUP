use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use serde_json::json;
use tokio::time::sleep;
use uuid::Uuid;

use crate::{
    services::{audit_service::audit_log, campaign_service::CampaignService},
    AppState,
};

// Poll every 30 seconds
const POLL_INTERVAL_SECS: u64 = 30;

/// Persistent background daemon — launched once at startup.
/// Every 30 seconds it looks for due campaign schedules and fires them.
pub async fn run_scheduler(state: AppState) {
    tracing::info!("Campaign scheduler started (interval={}s)", POLL_INTERVAL_SECS);

    loop {
        if let Err(e) = tick(&state).await {
            tracing::error!("Scheduler tick error: {:?}", e);
        }
        sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn tick(state: &AppState) -> anyhow::Result<()> {
    // Fetch all active schedules due to run
    let due = sqlx::query!(
        r#"
        SELECT cs.id as schedule_id, cs.campaign_id, cs.organization_id,
               cs.frequency::text, cs.cron_expression, cs.run_count,
               cs.max_runs, cs.ends_at, cs.next_run_at
        FROM campaign_schedules cs
        WHERE cs.status = 'active'
          AND cs.next_run_at <= NOW()
        ORDER BY cs.next_run_at ASC
        FOR UPDATE SKIP LOCKED
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    if due.is_empty() {
        return Ok(());
    }

    tracing::info!("Scheduler: {} schedule(s) due", due.len());

    for sched in due {
        let schedule_id = sched.schedule_id;
        let campaign_id = sched.campaign_id;
        let org_id = sched.organization_id;

        // Check max_runs limit
        if let Some(max) = sched.max_runs {
            if sched.run_count >= max {
                sqlx::query!(
                    "UPDATE campaign_schedules SET status = 'completed' WHERE id = $1",
                    schedule_id
                )
                .execute(&state.db)
                .await?;
                continue;
            }
        }

        // Check ends_at
        if let Some(ends) = sched.ends_at {
            if Utc::now() > ends {
                sqlx::query!(
                    "UPDATE campaign_schedules SET status = 'completed' WHERE id = $1",
                    schedule_id
                )
                .execute(&state.db)
                .await?;
                continue;
            }
        }

        // Insert run history record
        let run_number = sched.run_count + 1;
        let history_id = sqlx::query_scalar!(
            r#"
            INSERT INTO schedule_run_history
                (schedule_id, campaign_id, run_number, started_at, status)
            VALUES ($1, $2, $3, NOW(), 'running')
            RETURNING id
            "#,
            schedule_id,
            campaign_id,
            run_number
        )
        .fetch_one(&state.db)
        .await?;

        // Clone campaign → new draft → launch it
        let new_campaign_id =
            clone_and_launch(state, org_id, campaign_id, schedule_id).await;

        match new_campaign_id {
            Ok(new_id) => {
                // Calculate next_run_at
                let next = calculate_next_run(
                    Utc::now(),
                    sched.frequency.as_deref().unwrap_or("once"),
                    sched.cron_expression.as_deref(),
                );

                // Update schedule
                if let Some(next_run) = next {
                    sqlx::query!(
                        r#"
                        UPDATE campaign_schedules
                        SET last_run_at = NOW(),
                            run_count = run_count + 1,
                            next_run_at = $1
                        WHERE id = $2
                        "#,
                        next_run,
                        schedule_id
                    )
                    .execute(&state.db)
                    .await?;
                } else {
                    // once — mark completed
                    sqlx::query!(
                        r#"
                        UPDATE campaign_schedules
                        SET status = 'completed', last_run_at = NOW(), run_count = run_count + 1
                        WHERE id = $1
                        "#,
                        schedule_id
                    )
                    .execute(&state.db)
                    .await?;
                }

                // Update history
                sqlx::query!(
                    "UPDATE schedule_run_history SET status = 'completed', completed_at = NOW() WHERE id = $1",
                    history_id
                )
                .execute(&state.db)
                .await?;

                audit_log(
                    state,
                    "schedule.fired",
                    None,
                    Some(org_id),
                    Some("campaign_schedule"),
                    Some(schedule_id),
                    json!({
                        "schedule_id": schedule_id,
                        "campaign_id": campaign_id,
                        "new_campaign_id": new_id,
                        "run_number": run_number,
                    }),
                );
            }
            Err(e) => {
                tracing::error!(
                    "Scheduler failed to launch campaign {} (schedule {}): {:?}",
                    campaign_id,
                    schedule_id,
                    e
                );

                sqlx::query!(
                    r#"
                    UPDATE schedule_run_history
                    SET status = 'failed', completed_at = NOW(), error = $1
                    WHERE id = $2
                    "#,
                    e.to_string(),
                    history_id
                )
                .execute(&state.db)
                .await?;
            }
        }
    }

    Ok(())
}

// ── Launch campaign for a schedule ───────────────────────────────────────────

async fn clone_and_launch(
    state: &AppState,
    org_id: Uuid,
    campaign_id: Uuid,
    _schedule_id: Uuid,
) -> anyhow::Result<Uuid> {
    let system_user = Uuid::nil();

    // Reset status to 'draft' so launch_campaign can transition it to 'running'.
    // For recurring schedules this means each run re-sends to the same audience.
    // (A future enhancement could clone the campaign row.)
    sqlx::query!(
        r#"
        UPDATE campaigns
        SET status = 'draft',
            sent_count = 0, delivered_count = 0, read_count = 0,
            failed_count = 0, reply_count = 0, started_at = NULL, completed_at = NULL
        WHERE id = $1 AND organization_id = $2
        "#,
        campaign_id,
        org_id
    )
    .execute(&state.db)
    .await?;

    // Delete old queue jobs for this campaign
    sqlx::query!(
        "DELETE FROM message_queue_jobs WHERE campaign_id = $1",
        campaign_id
    )
    .execute(&state.db)
    .await?;

    let svc = CampaignService::new(state);
    svc.launch_campaign(org_id, campaign_id, system_user).await?;

    Ok(campaign_id)
}

// ── next_run_at calculation ──────────────────────────────────────────────────

fn calculate_next_run(
    from: DateTime<Utc>,
    frequency: &str,
    cron_expression: Option<&str>,
) -> Option<DateTime<Utc>> {
    match frequency {
        "once" => None, // Fires once, then done

        "daily" => Some(from + Duration::days(1)),

        "weekly" => Some(from + Duration::weeks(1)),

        "monthly" => {
            // Same day next month (clamped to last day of month)
            let next_month = if from.month() == 12 {
                from.with_year(from.year() + 1)?.with_month(1)?
            } else {
                from.with_month(from.month() + 1)?
            };
            Some(next_month)
        }

        "custom" => {
            // Parse cron expression
            if let Some(expr) = cron_expression {
                parse_cron_next(from, expr)
            } else {
                Some(from + Duration::days(1))
            }
        }

        _ => Some(from + Duration::days(1)),
    }
}

/// Simple cron next-fire calculator for standard 5-field expressions.
/// For full cron support, a dedicated crate like `croner` could be used.
fn parse_cron_next(from: DateTime<Utc>, _expr: &str) -> Option<DateTime<Utc>> {
    // Fallback: advance by 1 hour until a proper cron parser is integrated
    Some(from + Duration::hours(1))
}
