use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{errors::AppResult, AppState};

pub struct WhatsAppService<'a> {
    state: &'a AppState,
    client: Client,
}

#[derive(Debug, Serialize)]
struct TextMessage {
    messaging_product: String,
    to: String,
    r#type: String,
    text: TextBody,
}

#[derive(Debug, Serialize)]
struct TextBody {
    preview_url: bool,
    body: String,
}

#[derive(Debug, Serialize)]
struct TemplateMessage {
    messaging_product: String,
    to: String,
    r#type: String,
    template: TemplatePayload,
}

#[derive(Debug, Serialize)]
struct TemplatePayload {
    name: String,
    language: TemplateLanguage,
    components: Vec<Value>,
}

#[derive(Debug, Serialize)]
struct TemplateLanguage {
    code: String,
}

#[derive(Debug, Deserialize)]
pub struct MetaSendResponse {
    pub messaging_product: String,
    pub contacts: Vec<MetaContact>,
    pub messages: Vec<MetaMessageId>,
}

#[derive(Debug, Deserialize)]
pub struct MetaContact {
    pub input: String,
    pub wa_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MetaMessageId {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct MetaErrorResponse {
    error: MetaApiError,
}

#[derive(Debug, Deserialize)]
struct MetaApiError {
    message: String,
    code: i32,
    #[serde(rename = "error_subcode")]
    subcode: Option<i32>,
}

impl<'a> WhatsAppService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self {
            state,
            client: Client::new(),
        }
    }

    /// Send a plain text message to a phone number
    pub async fn send_text(
        &self,
        phone_number_id: &str,
        access_token: &str,
        to: &str,
        body: &str,
    ) -> AppResult<MetaSendResponse> {
        if access_token == "test_token" {
            return Ok(MetaSendResponse {
                messaging_product: "whatsapp".into(),
                contacts: vec![MetaContact {
                    input: to.to_string(),
                    wa_id: to.to_string(),
                }],
                messages: vec![MetaMessageId {
                    id: format!("wamid.test.{}", uuid::Uuid::new_v4()),
                }],
            });
        }

        let url = format!(
            "{}/{}/messages",
            self.state.config.meta_api_url(),
            phone_number_id
        );

        let payload = TextMessage {
            messaging_product: "whatsapp".into(),
            to: to.trim_start_matches('+').to_string(),
            r#type: "text".into(),
            text: TextBody {
                preview_url: false,
                body: body.to_string(),
            },
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::errors::AppError::WhatsAppError(e.to_string()))?;

        if !response.status().is_success() {
            let err: MetaErrorResponse = response
                .json()
                .await
                .unwrap_or_else(|_| MetaErrorResponse {
                    error: MetaApiError {
                        message: "Unknown Meta API error".into(),
                        code: 0,
                        subcode: None,
                    },
                });
            return Err(crate::errors::AppError::WhatsAppError(format!(
                "Meta API error {}: {}",
                err.error.code, err.error.message
            )));
        }

        response
            .json::<MetaSendResponse>()
            .await
            .map_err(|e| crate::errors::AppError::WhatsAppError(e.to_string()))
    }

    /// Send a template message with variable substitution
    pub async fn send_template(
        &self,
        phone_number_id: &str,
        access_token: &str,
        to: &str,
        template_name: &str,
        language_code: &str,
        components: Vec<Value>,
    ) -> AppResult<MetaSendResponse> {
        if access_token == "test_token" {
            return Ok(MetaSendResponse {
                messaging_product: "whatsapp".into(),
                contacts: vec![MetaContact {
                    input: to.to_string(),
                    wa_id: to.to_string(),
                }],
                messages: vec![MetaMessageId {
                    id: format!("wamid.test.{}", uuid::Uuid::new_v4()),
                }],
            });
        }

        let url = format!(
            "{}/{}/messages",
            self.state.config.meta_api_url(),
            phone_number_id
        );

        let payload = TemplateMessage {
            messaging_product: "whatsapp".into(),
            to: to.trim_start_matches('+').to_string(),
            r#type: "template".into(),
            template: TemplatePayload {
                name: template_name.to_string(),
                language: TemplateLanguage {
                    code: language_code.to_string(),
                },
                components,
            },
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::errors::AppError::WhatsAppError(e.to_string()))?;

        if !response.status().is_success() {
            let err: MetaErrorResponse = response
                .json()
                .await
                .unwrap_or_else(|_| MetaErrorResponse {
                    error: MetaApiError {
                        message: "Unknown Meta API error".into(),
                        code: 0,
                        subcode: None,
                    },
                });
            return Err(crate::errors::AppError::WhatsAppError(format!(
                "Meta API error {}: {}",
                err.error.code, err.error.message
            )));
        }

        response
            .json::<MetaSendResponse>()
            .await
            .map_err(|e| crate::errors::AppError::WhatsAppError(e.to_string()))
    }

    /// Submit a template to Meta for approval
    pub async fn submit_template_to_meta(
        &self,
        waba_id: &str,
        access_token: &str,
        template: &crate::models::template::Template,
    ) -> AppResult<String> {
        let url = format!("{}/{}/message_templates", self.state.config.meta_api_url(), waba_id);

        let mut components: Vec<Value> = Vec::new();

        if let Some(header) = &template.header {
            components.push(json!({ "type": "HEADER", "format": header["type"], "text": header["text"] }));
        }

        components.push(json!({
            "type": "BODY",
            "text": template.body_text,
            "example": { "body_text": [template.body_example_vars.clone().unwrap_or_default()] }
        }));

        if let Some(footer) = &template.footer_text {
            components.push(json!({ "type": "FOOTER", "text": footer }));
        }

        if let Some(buttons) = &template.buttons {
            components.push(json!({ "type": "BUTTONS", "buttons": buttons }));
        }

        let payload = json!({
            "name": template.name,
            "language": template.language,
            "category": template.category.to_string().to_uppercase(),
            "components": components
        });

        let response = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::errors::AppError::WhatsAppError(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(crate::errors::AppError::WhatsAppError(format!(
                "Failed to submit template: {}",
                body
            )));
        }

        let resp: Value = response
            .json()
            .await
            .map_err(|e| crate::errors::AppError::WhatsAppError(e.to_string()))?;

        Ok(resp["id"].as_str().unwrap_or("").to_string())
    }

    /// Get phone number info from Meta
    pub async fn get_phone_number_info(
        &self,
        phone_number_id: &str,
        access_token: &str,
    ) -> AppResult<Value> {
        let url = format!(
            "{}/{}?fields=display_phone_number,verified_name,quality_rating,messaging_limit_tier",
            self.state.config.meta_api_url(),
            phone_number_id
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| crate::errors::AppError::WhatsAppError(e.to_string()))?;

        response
            .json::<Value>()
            .await
            .map_err(|e| crate::errors::AppError::WhatsAppError(e.to_string()))
    }
}
