# Environment variables

See `backend/api/.env.example` and `frontend/.env.local.example`.

Required for local API:

- `DATABASE_URL` — Compose Postgres is `postgresql://postgres:postgres@localhost:5434/whatsup` unless you changed compose credentials
- `REDIS_URL`
- `JWT_SECRET`, `JWT_REFRESH_SECRET`
- `ENCRYPTION_KEY` — 64-char hex in production

Provider:

- `MESSAGING_PROVIDER=mock|meta` (default mock)
- `ENABLE_MOCK_PROVIDER=true`
- `MOCK_FAILURE_RATE=0.0`

Frontend:

- `NEXT_PUBLIC_API_URL=http://localhost:8080/api/v1`
