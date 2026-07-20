use chrono::{DateTime, Datelike, Duration, Utc};
use serde_json::json;
use tokio::time::sleep;
use uuid::Uuid;

use crate::{
    services::{audit_service::audit_log, campaign_service::CampaignService},
    AppState,
};

const POLL_INTERVAL_SECS: u64 = 30;

/// Persistent background daemon — launched once at startup.
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
    let mut tx = state.db.begin().await?;

    let due = sqlx::query!(
        r#"
        SELECT cs.id as schedule_id, cs.campaign_id, cs.organization_id,
               cs.frequency::text, cs.cron_expression, cs.run_count,
               cs.max_runs, cs.ends_at, cs.next_run_at
        FROM campaign_schedules cs
        WHERE cs.status = 'active'
          AND cs.next_run_at <= NOW()
        ORDER BY cs.next_run_at ASC
        FOR UPDATE OF cs SKIP LOCKED
        "#,
    )
    .fetch_all(&mut *tx)
    .await?;

    if due.is_empty() {
        tx.commit().await?;
        return Ok(());
    }

    // Claim by bumping next_run_at temporarily so other schedulers skip them
    let ids: Vec<Uuid> = due.iter().map(|s| s.schedule_id).collect();
    sqlx::query!(
        r#"
        UPDATE campaign_schedules
        SET next_run_at = NOW() + INTERVAL '1 hour'
        WHERE id = ANY($1)
        "#,
        &ids
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!("Scheduler: {} schedule(s) due", due.len());

    for sched in due {
        let schedule_id = sched.schedule_id;
        let campaign_id = sched.campaign_id;
        let org_id = sched.organization_id;

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

        // Clone campaign into a new draft, then launch (do not reset the template campaign)
        let new_campaign_id = clone_and_launch(state, org_id, campaign_id).await;

        match new_campaign_id {
            Ok(new_id) => {
                let next = calculate_next_run(
                    Utc::now(),
                    sched.frequency.as_deref().unwrap_or("once"),
                    sched.cron_expression.as_deref(),
                );

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

                // Restore original next_run so it can retry
                sqlx::query!(
                    "UPDATE campaign_schedules SET next_run_at = NOW() WHERE id = $1 AND status = 'active'",
                    schedule_id
                )
                .execute(&state.db)
                .await?;

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

/// Clone the source campaign to a new draft and launch it.
async fn clone_and_launch(
    state: &AppState,
    org_id: Uuid,
    campaign_id: Uuid,
) -> anyhow::Result<Uuid> {
    let system_user = Uuid::nil();
    let svc = CampaignService::new(state);
    let cloned = svc.clone_campaign(org_id, campaign_id, system_user).await?;
    let new_id = cloned["id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| anyhow::anyhow!("Clone did not return campaign id"))?;

    svc.launch_campaign(org_id, new_id, system_user).await?;
    Ok(new_id)
}

fn calculate_next_run(
    from: DateTime<Utc>,
    frequency: &str,
    cron_expression: Option<&str>,
) -> Option<DateTime<Utc>> {
    match frequency {
        "once" => None,
        "daily" => Some(from + Duration::days(1)),
        "weekly" => Some(from + Duration::weeks(1)),
        "monthly" => {
            let next_month = if from.month() == 12 {
                from.with_year(from.year() + 1)?.with_month(1)?
            } else {
                from.with_month(from.month() + 1)?
            };
            Some(next_month)
        }
        "custom" => {
            // Full cron parsing deferred; advance 1 hour as documented Phase 1 limitation
            let _ = cron_expression;
            Some(from + Duration::hours(1))
        }
        _ => Some(from + Duration::days(1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_has_no_next_run() {
        assert!(calculate_next_run(Utc::now(), "once", None).is_none());
    }

    #[test]
    fn daily_advances_one_day() {
        let from = Utc::now();
        let next = calculate_next_run(from, "daily", None).unwrap();
        assert!(next > from);
    }
}
