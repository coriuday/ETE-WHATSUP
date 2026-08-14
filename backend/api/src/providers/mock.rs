use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppResult;

use super::{
    MessagingProvider, OutboundMedia, OutboundTemplate, OutboundText, ProviderAccount, ProviderKind,
    SendResult,
};

pub struct MockWhatsAppProvider {
    pub simulate_failure_rate: f64,
}

impl Default for MockWhatsAppProvider {
    fn default() -> Self {
        Self {
            simulate_failure_rate: 0.0,
        }
    }
}

#[async_trait]
impl MessagingProvider for MockWhatsAppProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Mock
    }

    async fn send_message(&self, _account: &ProviderAccount, msg: OutboundText) -> AppResult<SendResult> {
        if self.should_fail() {
            return Err(crate::errors::AppError::WhatsAppError(
                "mock provider simulated failure".into(),
            ));
        }
        if msg.body.to_lowercase().contains("[fail]") {
            return Err(crate::errors::AppError::WhatsAppError(
                "mock provider forced failure".into(),
            ));
        }
        Ok(SendResult {
            provider_message_id: format!("mock:{}", Uuid::new_v4()),
            status: "sent".into(),
        })
    }

    async fn send_template(
        &self,
        account: &ProviderAccount,
        msg: OutboundTemplate,
    ) -> AppResult<SendResult> {
        self.send_message(
            account,
            OutboundText {
                to: msg.to,
                body: format!("template:{}", msg.template_name),
            },
        )
        .await
    }

    async fn send_media(&self, account: &ProviderAccount, msg: OutboundMedia) -> AppResult<SendResult> {
        self.send_message(
            account,
            OutboundText {
                to: msg.to,
                body: format!("media:{}", msg.media_url),
            },
        )
        .await
    }

    async fn get_message_status(&self, provider_message_id: &str) -> AppResult<String> {
        Ok(if provider_message_id.starts_with("mock:") {
            "sent".into()
        } else {
            "unknown".into()
        })
    }

    async fn process_webhook(&self, _payload: serde_json::Value) -> AppResult<()> {
        Ok(())
    }

    async fn health_check(&self, _account: &ProviderAccount) -> AppResult<String> {
        Ok("healthy".into())
    }
}

impl MockWhatsAppProvider {
    fn should_fail(&self) -> bool {
        self.simulate_failure_rate > 0.0 && rand::random::<f64>() < self.simulate_failure_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn account() -> ProviderAccount {
        ProviderAccount {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            kind: ProviderKind::Mock,
            phone_number_id: "mock".into(),
            access_token: String::new(),
            display_phone: None,
        }
    }

    #[tokio::test]
    async fn mock_send_succeeds() {
        let p = MockWhatsAppProvider::default();
        let r = p
            .send_message(
                &account(),
                OutboundText {
                    to: "+15551212".into(),
                    body: "hello".into(),
                },
            )
            .await
            .unwrap();
        assert!(r.provider_message_id.starts_with("mock:"));
        assert_eq!(r.status, "sent");
    }

    #[tokio::test]
    async fn mock_send_can_force_fail() {
        let p = MockWhatsAppProvider::default();
        let err = p
            .send_message(
                &account(),
                OutboundText {
                    to: "+15551212".into(),
                    body: "[fail] boom".into(),
                },
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("forced"));
    }
}
