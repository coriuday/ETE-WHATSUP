use crate::AppState;

use super::mock::MockWhatsAppProvider;
use super::{ProviderAccount, ProviderKind};

pub fn resolve_kind(state: &AppState, account_provider: Option<&str>, access_token: &str) -> ProviderKind {
    let configured = ProviderKind::parse(&state.config.messaging_provider);
    if configured == ProviderKind::Mock {
        return ProviderKind::Mock;
    }
    if let Some(p) = account_provider {
        let k = ProviderKind::parse(p);
        if k == ProviderKind::Mock {
            return ProviderKind::Mock;
        }
    }
    if access_token.is_empty() || access_token == "mock" {
        return ProviderKind::Mock;
    }
    ProviderKind::Meta
}

pub fn mock_provider(state: &AppState) -> MockWhatsAppProvider {
    MockWhatsAppProvider {
        simulate_failure_rate: state.config.mock_failure_rate,
    }
}

pub fn account_from_row(
    id: uuid::Uuid,
    organization_id: uuid::Uuid,
    kind: ProviderKind,
    phone_number_id: Option<String>,
    access_token: String,
    display_phone: Option<String>,
) -> ProviderAccount {
    ProviderAccount {
        id,
        organization_id,
        kind,
        phone_number_id: phone_number_id.unwrap_or_else(|| "mock".into()),
        access_token,
        display_phone,
    }
}
