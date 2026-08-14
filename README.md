# WhatsUp — Enterprise WhatsApp Bulk Messaging & Automation Platform

A multi-tenant, secure, high-performance SaaS platform built to handle high-volume WhatsApp broadcasting, real-time customer communications, automated follow-up sequences, and detailed campaign delivery funnels.

## 🚀 Technology Stack
*   **Frontend**: Next.js 15, TypeScript, Tailwind CSS, Zustand, TanStack Query, Recharts
*   **Backend**: Rust, Axum, SQLx (PostgreSQL), Redis (rate limit + auth cache), JWT + RBAC, Lettre (SMTP)
*   **Database**: PostgreSQL (Docker Compose locally; Supabase compatible)
*   **Job queue**: Postgres `message_queue_jobs` (in-process workers)
*   **Automation**: self-hosted n8n
*   **Storage**: S3-Compatible Storage (MinIO locally, AWS S3 / Cloudflare R2 in production)
*   **Proxying**: Nginx Reverse Proxy
*   **Infrastructure**: Docker & Docker Compose

---

## 📁 Repository Structure
```
ete_whatsup/
├── backend/
│   └── api/                # Axum REST API in Rust (lib + binary)
├── frontend/               # Next.js 15 Dashboard & Auth
├── db/
│   └── migrations/         # Database migrations (001 – 017)
├── infrastructure/
│   ├── nginx/              # Nginx reverse proxy configuration
│   ├── redis/              # Redis server configuration
│   └── n8n/                # n8n automations configuration
└── docker-compose.yml      # Local stack (API, frontend, postgres, redis, minio, n8n, nginx)
```

---

## Multi-tenant API notes
* Org-scoped routes require header `X-Organization-Id` (UUID of the active organization).
* Users with a single membership may omit the header (fallback); multi-org users must send it.
* Health: `GET /api/v1/health/live` (liveness), `GET /api/v1/health/ready` (DB + Redis).
* Migrations are applied externally (Compose init / ops) — not auto-run on API boot.
---

## 🛠️ Local Development Setup

### Prerequisites
1.  **Docker & Docker Compose** installed.
2.  **Bun (v1.1+)** (for local frontend execution & package management).
3.  **Rust & Cargo (v1.75+)** (for local backend compilation).

### Step 1: Configure Environments
Copy the environment template files and fill in values (such as your Supabase database string and Meta WABA tokens):

1.  **Backend**:
    ```bash
    cp backend/api/.env.example backend/api/.env
    ```
2.  **Frontend**:
    ```bash
    cp frontend/.env.local.example frontend/.env.local
    ```

### Step 2: Launch Docker Compose Services
Start postgres, caching, storage, proxy, and automation containers:
```bash
docker compose up -d
```
This boots up:
*   **Nginx Proxy** on `http://localhost:80`
*   **Next.js Frontend** on `http://localhost:3000`
*   **Axum Backend API** on `http://localhost:8080`
*   **PostgreSQL** (Compose service)
*   **n8n Workflow Editor** on `http://localhost:5678`
*   **MinIO Console** on `http://localhost:9001`
*   **Redis** on `http://localhost:6379` (rate limiting + auth session cache — not the message job queue)

### Step 3: Run Database Migrations
Apply SQL files in order (`db/migrations/001` … `017`) against your Postgres database.
```bash
# Example with psql against Compose postgres
psql "$DATABASE_URL" -f db/migrations/001_....sql
# … through 015_phase1_stabilization.sql
```

---

## 🔒 Security Configuration
*   All WhatsApp Account access tokens and credentials are encrypted on write using **AES-256-GCM** via the backend `encryption.rs` service.
*   Two-Factor Authentication uses secure **Time-Based One-Time Passwords (TOTP)** verified with authenticator apps.
*   Strict Role-Based Access Control (RBAC) categorizes users into:
    1.  `Super Admin`: System monitoring, org subscriptions management.
    2.  `Business Admin`: Workspace adjustments, member invitations, billing.
    3.  `Team Member`: Create campaign drafts, message inbox chats.
*   For details on vulnerability disclosure and patches, see [SECURITY.md](SECURITY.md).

---

## 🤝 Open Source & Collaboration
We welcome outside improvements and bug fixes! Please read our guidelines to get started:
*   [Contributing Guidelines](CONTRIBUTING.md) — standards for PR branches, code quality, and testing.
*   [Code of Conduct](CODE_OF_CONDUCT.md) — professional behavior guidelines.
*   [MIT License](LICENSE) — licensing terms for open source use.

