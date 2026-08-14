# Mock WhatsApp

Enable with:

```
MESSAGING_PROVIDER=mock
ENABLE_MOCK_PROVIDER=true
```

Authenticated routes (org-scoped):

- `POST /api/v1/dev/mock/inbound` `{ "phone_number", "body", "contact_name?" }`
- `POST /api/v1/dev/mock/status` `{ "message_id", "status" }` where status is `delivered` | `read` | `failed` | `sent` | `processing` | `retrying` | `cancelled`

These write real contacts, conversations, messages, campaign counters, and can fire automations.

In the UI: Settings → Developer — Simulate inbound. WhatsApp → Add mock account.

Force a send failure by including `[fail]` in the message body.
