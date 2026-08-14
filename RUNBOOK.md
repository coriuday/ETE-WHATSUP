# Local Development Runbook

Preferred inner loop: Compose for data stores, Cargo + Next on the host.

## 1. Infrastructure

```powershell
docker compose up -d postgres redis minio n8n
```

Postgres is published on **5434**. Redis stays on the Docker network unless you map a port.

## 2. Environment

```powershell
cp backend/api/.env.example backend/api/.env
cp frontend/.env.local.example frontend/.env.local
```

Point `DATABASE_URL` at `localhost:5434`. Set:

```
APP_PORT=8080
MESSAGING_PROVIDER=mock
ENABLE_MOCK_PROVIDER=true
NEXT_PUBLIC_API_URL=http://localhost:8080/api/v1
```

Apply `db/migrations/*.sql` in order (001–017).

## 3. API

```powershell
cd backend/api
cargo run --bin whatsup-api
```

Listens on **8080** by default.

## 4. Frontend

```powershell
cd frontend
bun run dev
```

Open http://localhost:3000

See also `docs/mock-whatsapp.md` and `docs/env.md`.
