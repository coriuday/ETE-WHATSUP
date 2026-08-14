use sqlx::Row;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::AppState;

use super::factory::{account_from_row, mock_provider, resolve_kind};
use super::meta::MetaWhatsAppProvider;
use super::{
    MessagingProvider, OutboundMedia, OutboundTemplate, OutboundText, ProviderAccount, ProviderKind,
    SendResult,
};

async fn row_to_account(
    state: &AppState,
    id: Uuid,
    organization_id: Uuid,
    provider: Option<String>,
    phone_number_id: Option<String>,
    access_token_enc: Option<String>,
    phone_number: String,
) -> ProviderAccount {
    let token = match access_token_enc {
        Some(enc) => {
            crate::utils::encryption::decrypt(&enc, &state.config.encryption_key).unwrap_or_default()
        }
        None => String::new(),
    };
    let kind = resolve_kind(state, provider.as_deref(), &token);
    account_from_row(
        id,
        organization_id,
        kind,
        phone_number_id,
        token,
        Some(phone_number),
    )
}

pub async fn load_account(state: &AppState, wa_account_id: Uuid, org_id: Uuid) -> AppResult<ProviderAccount> {
    let row = sqlx::query(
        r#"
        SELECT id, organization_id, provider, phone_number_id, access_token_enc, phone_number
        FROM whatsapp_accounts
        WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(wa_account_id)
    .bind(org_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::WaNotConnected)?;

    Ok(row_to_account(
        state,
        row.get("id"),
        row.get("organization_id"),
        row.try_get("provider").ok(),
        row.try_get("phone_number_id").ok(),
        row.try_get("access_token_enc").ok(),
        row.get("phone_number"),
    )
    .await)
}

pub async fn load_account_for_campaign(state: &AppState, campaign_id: Uuid) -> AppResult<ProviderAccount> {
    let row = sqlx::query(
        r#"
        SELECT wa.id, wa.organization_id, wa.provider, wa.phone_number_id, wa.access_token_enc, wa.phone_number
        FROM whatsapp_accounts wa
        JOIN campaigns c ON c.wa_account_id = wa.id
        WHERE c.id = $1
        "#,
    )
    .bind(campaign_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("WhatsApp account for campaign".into()))?;

    Ok(row_to_account(
        state,
        row.get("id"),
        row.get("organization_id"),
        row.try_get("provider").ok(),
        row.try_get("phone_number_id").ok(),
        row.try_get("access_token_enc").ok(),
        row.get("phone_number"),
    )
    .await)
}

pub async fn ensure_mock_account(state: &AppState, org_id: Uuid) -> AppResult<ProviderAccount> {
    if let Some(row) = sqlx::query(
        r#"
        SELECT id, organization_id, provider, phone_number_id, access_token_enc, phone_number
        FROM whatsapp_accounts
        WHERE organization_id = $1 AND deleted_at IS NULL
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(org_id)
    .fetch_optional(&state.db)
    .await?
    {
        return Ok(row_to_account(
            state,
            row.get("id"),
            row.get("organization_id"),
            row.try_get("provider").ok(),
            row.try_get("phone_number_id").ok(),
            row.try_get("access_token_enc").ok(),
            row.get("phone_number"),
        )
        .await);
    }

    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO whatsapp_accounts (
            organization_id, display_name, phone_number, phone_number_id, status, provider
        ) VALUES ($1, 'Mock WhatsApp', '+15550000000', 'mock', 'connected', 'mock')
        RETURNING id
        "#,
    )
    .bind(org_id)
    .fetch_one(&state.db)
    .await?;

    Ok(account_from_row(
        id,
        org_id,
        ProviderKind::Mock,
        Some("mock".into()),
        String::new(),
        Some("+15550000000".into()),
    ))
}

pub async fn send_text(state: &AppState, account: &ProviderAccount, msg: OutboundText) -> AppResult<SendResult> {
    match account.kind {
        ProviderKind::Mock => mock_provider(state).send_message(account, msg).await,
        ProviderKind::Meta => MetaWhatsAppProvider::new(state).send_message(account, msg).await,
    }
}

pub async fn send_template(
    state: &AppState,
    account: &ProviderAccount,
    msg: OutboundTemplate,
) -> AppResult<SendResult> {
    match account.kind {
        ProviderKind::Mock => mock_provider(state).send_template(account, msg).await,
        ProviderKind::Meta => MetaWhatsAppProvider::new(state).send_template(account, msg).await,
    }
}

pub async fn send_media(
    state: &AppState,
    account: &ProviderAccount,
    msg: OutboundMedia,
) -> AppResult<SendResult> {
    match account.kind {
        ProviderKind::Mock => mock_provider(state).send_media(account, msg).await,
        ProviderKind::Meta => MetaWhatsAppProvider::new(state).send_media(account, msg).await,
    }
}

pub async fn health_check(state: &AppState, account: &ProviderAccount) -> AppResult<String> {
    match account.kind {
        ProviderKind::Mock => mock_provider(state).health_check(account).await,
        ProviderKind::Meta => MetaWhatsAppProvider::new(state).health_check(account).await,
    }
}
