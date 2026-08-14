use serde_json::{json, Value};
use uuid::Uuid;

use crate::AppState;

pub async fn dispatch(state: &AppState, org_id: Uuid, trigger_type: &str, payload: Value) {
    let workflows = sqlx::query_as::<_, (Uuid, String, Value)>(
        r#"
        SELECT id, name, definition
        FROM automation_workflows
        WHERE organization_id = $1
          AND deleted_at IS NULL
          AND enabled = TRUE
          AND trigger_type = $2
        "#,
    )
    .bind(org_id)
    .bind(trigger_type)
    .fetch_all(&state.db)
    .await;

    let Ok(workflows) = workflows else {
        return;
    };

    for (id, name, definition) in workflows {
        let run_id = Uuid::new_v4();
        let _ = sqlx::query(
            r#"
            INSERT INTO automation_runs (id, organization_id, workflow_id, status, trigger_type, payload)
            VALUES ($1, $2, $3, 'running', $4, $5)
            "#,
        )
        .bind(run_id)
        .bind(org_id)
        .bind(id)
        .bind(trigger_type)
        .bind(&payload)
        .execute(&state.db)
        .await;

        let result = execute_definition(state, org_id, &definition, &payload).await;
        match result {
            Ok(()) => {
                let _ = sqlx::query(
                    "UPDATE automation_runs SET status = 'completed', finished_at = NOW() WHERE id = $1",
                )
                .bind(run_id)
                .execute(&state.db)
                .await;
                tracing::info!("Automation {} ({}) completed", name, id);
            }
            Err(e) => {
                let _ = sqlx::query(
                    "UPDATE automation_runs SET status = 'failed', error = $2, finished_at = NOW() WHERE id = $1",
                )
                .bind(run_id)
                .bind(e.to_string())
                .execute(&state.db)
                .await;
                tracing::warn!("Automation {} failed: {}", name, e);
            }
        }
    }
}

async fn execute_definition(
    state: &AppState,
    org_id: Uuid,
    definition: &Value,
    payload: &Value,
) -> anyhow::Result<()> {
    let steps = definition["steps"].as_array().cloned().unwrap_or_default();
    for step in steps {
        let kind = step["type"].as_str().unwrap_or("");
        match kind {
            "condition" => {
                if !eval_condition(&step, payload) {
                    break;
                }
            }
            "add_tag" => {
                if let (Some(contact_id), Some(tag)) = (
                    payload["contact_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
                    step["tag"].as_str(),
                ) {
                    let _ = sqlx::query(
                        "UPDATE contacts SET tags = array_append(tags, $3) WHERE id = $1 AND organization_id = $2 AND NOT ($3 = ANY(tags))",
                    )
                    .bind(contact_id)
                    .bind(org_id)
                    .bind(tag)
                    .execute(&state.db)
                    .await;
                }
            }
            "remove_tag" => {
                if let (Some(contact_id), Some(tag)) = (
                    payload["contact_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
                    step["tag"].as_str(),
                ) {
                    let _ = sqlx::query(
                        "UPDATE contacts SET tags = array_remove(tags, $3) WHERE id = $1 AND organization_id = $2",
                    )
                    .bind(contact_id)
                    .bind(org_id)
                    .bind(tag)
                    .execute(&state.db)
                    .await;
                }
            }
            "webhook" | "http_request" | "n8n" => {
                if let Some(url) = step["url"].as_str() {
                    let _ = state
                        .http
                        .post(url)
                        .json(&json!({ "org_id": org_id, "payload": payload, "step": step }))
                        .send()
                        .await;
                }
            }
            "wait" => {
                let secs = step["seconds"].as_u64().unwrap_or(0).min(5);
                if secs > 0 {
                    tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                }
            }
            "assign_agent" => {
                if let (Some(conv_id), Some(user_id)) = (
                    payload["conversation_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok()),
                    step["user_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
                ) {
                    let _ = sqlx::query(
                        "UPDATE conversations SET assigned_to = $3, assigned_at = NOW() WHERE id = $1 AND organization_id = $2",
                    )
                    .bind(conv_id)
                    .bind(org_id)
                    .bind(user_id)
                    .execute(&state.db)
                    .await;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn eval_condition(step: &Value, payload: &Value) -> bool {
    let field = step["field"].as_str().unwrap_or("");
    let op = step["op"].as_str().unwrap_or("eq");
    let expected = step["value"].as_str().unwrap_or("");
    let actual = payload
        .pointer(field)
        .and_then(|v| v.as_str())
        .or_else(|| payload[field.trim_start_matches('/')].as_str())
        .unwrap_or("");
    match op {
        "contains" => actual.to_lowercase().contains(&expected.to_lowercase()),
        "exists" => !actual.is_empty(),
        _ => actual == expected,
    }
}
