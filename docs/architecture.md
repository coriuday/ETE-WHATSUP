# Architecture

WhatsUp is a multi-tenant messaging SaaS. The Next.js app talks only to the Axum API. The API owns contacts, campaigns, conversations, analytics, automations, and provider selection.

```
Frontend (Next.js)
  -> REST /api/v1
     -> PostgreSQL (source of truth)
     -> Redis (rate limit + auth cache)
     -> message_queue_jobs worker (same process)
        -> MessagingProvider (mock | meta)
     -> MinIO/S3 (CSV import media)
     -> n8n (optional HTTP action)
```

Tenant isolation: JWT + `X-Organization-Id` + org membership. Handlers must filter by `organization_id` from auth context, not from untrusted body fields.

Message status is backend-owned (`queued`, `processing`, `sent`, `delivered`, `read`, `failed`, `cancelled`, `retrying`). The UI only displays it.
