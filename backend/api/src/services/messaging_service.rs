/// Messaging service — re-exports and future inbox/outbox helpers.
/// The actual per-campaign sending lives in `services/worker/mod.rs`.
/// Conversation-level sending uses `WhatsAppService` directly.
pub use crate::services::whatsapp_service::WhatsAppService;
