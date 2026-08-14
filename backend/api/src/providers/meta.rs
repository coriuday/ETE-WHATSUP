use async_trait::async_trait;

use crate::errors::{AppError, AppResult};
use crate::services::whatsapp_service::WhatsAppService;
use crate::AppState;

use super::{
    MessagingProvider, OutboundMedia, OutboundTemplate, OutboundText, ProviderAccount, ProviderKind,
    SendResult,
};

pub struct MetaWhatsAppProvider<'a> {
    inner: WhatsAppService<'a>,
}

impl<'a> MetaWhatsAppProvider<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self {
            inner: WhatsAppService::new(state),
        }
    }
}

#[async_trait]
impl MessagingProvider for MetaWhatsAppProvider<'_> {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Meta
    }

    async fn send_message(&self, account: &ProviderAccount, msg: OutboundText) -> AppResult<SendResult> {
        let resp = self
            .inner
            .send_text(
                &account.phone_number_id,
                &account.access_token,
                &msg.to,
                &msg.body,
            )
            .await?;
        Ok(SendResult {
            provider_message_id: resp
                .messages
                .first()
                .map(|m| m.id.clone())
                .unwrap_or_default(),
            status: "sent".into(),
        })
    }

    async fn send_template(
        &self,
        account: &ProviderAccount,
        msg: OutboundTemplate,
    ) -> AppResult<SendResult> {
        let resp = self
            .inner
            .send_template(
                &account.phone_number_id,
                &account.access_token,
                &msg.to,
                &msg.template_name,
                &msg.language,
                msg.components,
            )
            .await?;
        Ok(SendResult {
            provider_message_id: resp
                .messages
                .first()
                .map(|m| m.id.clone())
                .unwrap_or_default(),
            status: "sent".into(),
        })
    }

    async fn send_media(
        &self,
        account: &ProviderAccount,
        msg: OutboundMedia,
    ) -> AppResult<SendResult> {
        // Alpha: send media URL as a captioned text until dedicated Graph media upload is wired.
        self.send_message(
            account,
            OutboundText {
                to: msg.to,
                body: format!(
                    "{}\n{}",
                    msg.caption.unwrap_or_default(),
                    msg.media_url
                )
                .trim()
                .to_string(),
            },
        )
        .await
    }

    async fn get_message_status(&self, _provider_message_id: &str) -> AppResult<String> {
        Ok("unknown".into())
    }

    async fn process_webhook(&self, _payload: serde_json::Value) -> AppResult<()> {
        Ok(())
    }

    async fn health_check(&self, account: &ProviderAccount) -> AppResult<String> {
        if account.access_token.is_empty() || account.phone_number_id.is_empty() {
            return Err(AppError::WaNotConnected);
        }
        match self
            .inner
            .get_phone_number_info(&account.phone_number_id, &account.access_token)
            .await
        {
            Ok(_) => Ok("healthy".into()),
            Err(e) => Err(e),
        }
    }
}
