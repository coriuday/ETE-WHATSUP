use anyhow::{bail, Context, Result};
use serde_json::json;
use uuid::Uuid;

use crate::{utils::phone::normalize_phone, AppState};

/// Row parsed from a CSV or XLSX file
#[derive(Debug, Default)]
struct ImportRow {
    phone_number: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    tags: Vec<String>,
}

/// Entry point — spawned from `ContactService::start_import()`.
/// Downloads the file, parses it, bulk-inserts contacts, and updates
/// the `contact_imports` row throughout.
pub async fn process_import(
    state: AppState,
    job_id: Uuid,
    org_id: Uuid,
    user_id: Uuid,
    file_url: String,
    filename: String,
) {
    if let Err(e) = run_import(&state, job_id, org_id, user_id, &file_url, &filename).await {
        tracing::error!("Import job {} failed: {:?}", job_id, e);
        let _ = sqlx::query!(
            "UPDATE contact_imports SET status = 'failed', error_details = $1 WHERE id = $2",
            json!([{ "error": e.to_string() }]),
            job_id
        )
        .execute(&state.db)
        .await;
    }
}

async fn run_import(
    state: &AppState,
    job_id: Uuid,
    org_id: Uuid,
    user_id: Uuid,
    file_url: &str,
    filename: &str,
) -> Result<()> {
    // 1. Mark as processing
    sqlx::query!(
        "UPDATE contact_imports SET status = 'processing' WHERE id = $1",
        job_id
    )
    .execute(&state.db)
    .await
    .context("Failed to update import status to processing")?;

    // 2. Download the file bytes
    let bytes = download_file(state, file_url).await?;

    // 3. Parse rows by extension
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("csv")
        .to_lowercase();

    let rows = match ext.as_str() {
        "xlsx" | "xls" => parse_xlsx(&bytes)?,
        _ => parse_csv(&bytes)?,
    };

    let total = rows.len() as i32;

    sqlx::query!(
        "UPDATE contact_imports SET total_rows = $1 WHERE id = $2",
        total,
        job_id
    )
    .execute(&state.db)
    .await
    .context("Failed to set total_rows")?;

    // 4. Process in batches of 1000
    const BATCH_SIZE: usize = 1000;
    let mut imported = 0i32;
    let mut skipped = 0i32;
    let mut error_count = 0i32;
    let mut error_details: Vec<serde_json::Value> = Vec::new();

    for (batch_idx, chunk) in rows.chunks(BATCH_SIZE).enumerate() {
        let base_row = (batch_idx * BATCH_SIZE) as i32;

        for (i, row) in chunk.iter().enumerate() {
            let row_num = base_row + i as i32 + 1;

            // Normalize phone
            let phone = match normalize_phone(&row.phone_number) {
                Some(p) => p,
                None => {
                    error_count += 1;
                    error_details.push(json!({
                        "row": row_num,
                        "phone": row.phone_number,
                        "error": "Invalid phone number format"
                    }));
                    continue;
                }
            };

            let tags: Vec<String> = row.tags.clone();

            let result = sqlx::query!(
                r#"
                INSERT INTO contacts
                    (organization_id, phone_number, first_name, last_name, email, tags,
                     source, created_by)
                VALUES ($1, $2, $3, $4, $5, $6, 'csv_import', $7)
                ON CONFLICT (organization_id, phone_number) DO UPDATE
                    SET first_name = COALESCE(EXCLUDED.first_name, contacts.first_name),
                        last_name  = COALESCE(EXCLUDED.last_name,  contacts.last_name),
                        email      = COALESCE(EXCLUDED.email,       contacts.email)
                "#,
                org_id,
                phone,
                row.first_name.as_deref(),
                row.last_name.as_deref(),
                row.email.as_deref(),
                &tags,
                user_id
            )
            .execute(&state.db)
            .await;

            match result {
                Ok(r) if r.rows_affected() > 0 => imported += 1,
                Ok(_) => skipped += 1,
                Err(e) => {
                    error_count += 1;
                    error_details.push(json!({
                        "row": row_num,
                        "phone": row.phone_number,
                        "error": e.to_string()
                    }));
                }
            }
        }

        // Update progress after each batch
        let processed_so_far = base_row + chunk.len() as i32;
        let _ = sqlx::query!(
            r#"
            UPDATE contact_imports
            SET processed_rows = $1,
                imported_count = $2,
                skipped_count  = $3,
                error_count    = $4
            WHERE id = $5
            "#,
            processed_so_far,
            imported,
            skipped,
            error_count,
            job_id
        )
        .execute(&state.db)
        .await;
    }

    // 5. Finalize
    sqlx::query!(
        r#"
        UPDATE contact_imports
        SET status        = 'completed',
            total_rows    = $1,
            processed_rows = $1,
            imported_count = $2,
            skipped_count  = $3,
            error_count    = $4,
            error_details  = $5,
            completed_at   = NOW()
        WHERE id = $6
        "#,
        total,
        imported,
        skipped,
        error_count,
        serde_json::Value::Array(error_details),
        job_id
    )
    .execute(&state.db)
    .await
    .context("Failed to finalize import job")?;

    tracing::info!(
        "Import job {} completed: {} imported, {} skipped, {} errors",
        job_id, imported, skipped, error_count
    );

    Ok(())
}

// ── File download ────────────────────────────────────────────────────────────

async fn download_file(state: &AppState, file_url: &str) -> Result<bytes::Bytes> {
    use crate::services::storage_service::StorageService;

    // Extract the object key from the URL
    // URL format: https://<endpoint>/<bucket>/<key>
    let key = extract_s3_key(file_url, &state.config.s3_bucket)
        .unwrap_or_else(|| file_url.to_string());

    let storage = StorageService::new(state);
    storage
        .download_bytes(&key)
        .await
        .context("Failed to download import file from storage")
}

fn extract_s3_key(url: &str, bucket: &str) -> Option<String> {
    // Find the bucket name in URL and take everything after it
    let needle = format!("/{}/", bucket);
    url.find(&needle).map(|pos| url[pos + needle.len()..].to_string())
}

// ── CSV Parsing ──────────────────────────────────────────────────────────────

fn parse_csv(data: &[u8]) -> Result<Vec<ImportRow>> {
    use csv::ReaderBuilder;

    let mut rdr = ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(data);

    let headers = rdr
        .headers()
        .context("Failed to read CSV headers")?
        .clone();

    // Normalize header names
    let headers: Vec<String> = headers
        .iter()
        .map(|h| h.trim().to_lowercase().replace(' ', "_"))
        .collect();

    let mut rows = Vec::new();

    for result in rdr.records() {
        let record = result.context("Failed to read CSV record")?;
        let mut row = ImportRow::default();

        for (i, field) in record.iter().enumerate() {
            let field = field.trim();
            if field.is_empty() {
                continue;
            }
            let header = headers.get(i).map(|s| s.as_str()).unwrap_or("");
            match header {
                "phone" | "phone_number" | "mobile" | "mobile_number" | "number" => {
                    row.phone_number = field.to_string();
                }
                "first_name" | "firstname" | "first" => {
                    row.first_name = Some(field.to_string());
                }
                "last_name" | "lastname" | "last" | "surname" => {
                    row.last_name = Some(field.to_string());
                }
                "name" | "full_name" | "fullname" => {
                    let parts: Vec<&str> = field.splitn(2, ' ').collect();
                    row.first_name = Some(parts[0].to_string());
                    if parts.len() > 1 {
                        row.last_name = Some(parts[1].to_string());
                    }
                }
                "email" | "email_address" => {
                    row.email = Some(field.to_string());
                }
                "tags" | "tag" | "labels" => {
                    row.tags = field
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect();
                }
                _ => {}
            }
        }

        if !row.phone_number.is_empty() {
            rows.push(row);
        }
    }

    Ok(rows)
}

// ── XLSX Parsing ─────────────────────────────────────────────────────────────

fn parse_xlsx(data: &[u8]) -> Result<Vec<ImportRow>> {
    use calamine::{open_workbook_from_rs, Reader, Xlsx};
    use std::io::Cursor;

    let cursor = Cursor::new(data.to_vec());
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .context("Failed to open XLSX file")?;

    let sheet_names = workbook.sheet_names().to_vec();
    let sheet_name = sheet_names
        .first()
        .context("No sheets found in XLSX file")?
        .clone();

    let range = workbook
        .worksheet_range(&sheet_name)
        .context("Failed to read worksheet")?;

    let mut rows_iter = range.rows();

    // First row = headers
    let header_row = match rows_iter.next() {
        Some(r) => r,
        None => return Ok(vec![]),
    };

    let headers: Vec<String> = header_row
        .iter()
        .map(|c| {
            c.to_string()
                .trim()
                .to_lowercase()
                .replace(' ', "_")
        })
        .collect();

    let mut rows = Vec::new();

    for data_row in rows_iter {
        let mut row = ImportRow::default();

        for (i, cell) in data_row.iter().enumerate() {
            let field = cell.to_string();
            let field = field.trim();
            if field.is_empty() {
                continue;
            }
            let header = headers.get(i).map(|s| s.as_str()).unwrap_or("");
            match header {
                "phone" | "phone_number" | "mobile" | "mobile_number" | "number" => {
                    row.phone_number = field.to_string();
                }
                "first_name" | "firstname" | "first" => {
                    row.first_name = Some(field.to_string());
                }
                "last_name" | "lastname" | "last" | "surname" => {
                    row.last_name = Some(field.to_string());
                }
                "name" | "full_name" | "fullname" => {
                    let parts: Vec<&str> = field.splitn(2, ' ').collect();
                    row.first_name = Some(parts[0].to_string());
                    if parts.len() > 1 {
                        row.last_name = Some(parts[1].to_string());
                    }
                }
                "email" | "email_address" => {
                    row.email = Some(field.to_string());
                }
                "tags" | "tag" | "labels" => {
                    row.tags = field
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect();
                }
                _ => {}
            }
        }

        if !row.phone_number.is_empty() {
            rows.push(row);
        }
    }

    Ok(rows)
}
