# Provider architecture

Business logic depends on `whatsup_api::providers::MessagingProvider`, not Meta Graph calls.

Implementations:

- `MockWhatsAppProvider` — default when `MESSAGING_PROVIDER=mock` or the account has no live token
- `MetaWhatsAppProvider` — wraps the existing Graph client

Factory: `providers::dispatch::{load_account, send_text, send_template, send_media, health_check}`

Campaign workers and inbox send both go through this layer. A workspace without Meta credentials still sends via mock and updates real `messages` rows.
