# Testing

Backend (from `backend/api`):

```bash
cargo test
```

Covers HMAC, org header, scheduler delays, worker retry ordering, mock provider send/fail, message transition rules. HTTP/DB integration tests need a running Postgres; CI currently runs unit tests without infra.

Frontend:

```bash
cd frontend
npm test
```

Vitest covers design-system primitives and campaign wizard copy.

Alpha integration path (manual or scripted against a running stack):

1. Register / login / create org
2. Add mock WhatsApp account
3. Create contact
4. Create + launch campaign
5. `POST /dev/mock/status` to delivered/read
6. `POST /dev/mock/inbound` and confirm Inbox + analytics
