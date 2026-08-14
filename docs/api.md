# API (high level)

Prefix: `/api/v1`. JSON. Org header: `X-Organization-Id`.

| Area | Paths |
|---|---|
| Health | `/health`, `/health/live`, `/health/ready` |
| Auth | `/auth/register`, `/login`, `/refresh`, `/me`, `/2fa/*` |
| Orgs | `/organizations`, `/:id/members`, `/:id/usage`, `/:id/audit` |
| Contacts | `/contacts`, `/export`, `/import`, `/groups`, `/segments` |
| Campaigns | `/campaigns`, `/:id/launch`, `/:id/schedule`, pause/resume/cancel |
| Inbox | `/conversations`, `/:id/messages`, assign/resolve/reopen/notes |
| WhatsApp | `/whatsapp/accounts`, `/:id/health` |
| Automations | `/automations`, `/automations/runs` |
| Analytics | `/analytics/overview`, `/campaigns`, `/messages`, `/contacts` |
| Mock | `/dev/mock/inbound`, `/dev/mock/status` |

Do not invent message statuses in the client; read `status` from the API.
