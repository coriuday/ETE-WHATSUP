# WhatsUp — Enterprise WhatsApp Bulk Messaging & Automation Platform

A multi-tenant, secure, high-performance SaaS platform built to handle high-volume WhatsApp broadcasting, real-time customer communications, automated follow-up sequences, and detailed campaign delivery funnels.

## 🚀 Technology Stack
*   **Frontend**: Next.js 15, TypeScript, Tailwind CSS, Zustand, TanStack Query, Recharts, Framer Motion
*   **Backend**: Rust, Axum, SQLx (PostgreSQL), Redis (Cache & Queue), JWT + RBAC, Lettre (SMTP client)
*   **Database**: Supabase PostgreSQL
*   **Automation**: self-hosted n8n
*   **Storage**: S3-Compatible Storage (MinIO locally, AWS S3 / Cloudflare R2 in production)
*   **Proxying**: Nginx Reverse Proxy
*   **Infrastructure**: Docker & Docker Compose

---

## 📁 Repository Structure
```
ete_whatsup/
├── backend/
│   └── api/                # Axum REST API in Rust
├── frontend/               # Next.js 15 Dashboard & Auth
├── db/
│   └── migrations/         # Database migrations (001 - 010)
├── infrastructure/
│   ├── nginx/              # Nginx reverse proxy configuration
│   ├── redis/              # Redis server configuration
│   └── n8n/                # n8n automations configuration
└── docker-compose.yml      # Local dev multi-container stack
```

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
Start the caching, storage, proxy, and automation containers:
```bash
docker compose up -d
```
This boots up:
*   **Nginx Proxy** on `http://localhost:80`
*   **Next.js Frontend** on `http://localhost:3000`
*   **Axum Backend API** on `http://localhost:8080`
*   **n8n Workflow Editor** on `http://localhost:5678`
*   **MinIO Console** on `http://localhost:9001`
*   **Redis Cache** on `http://localhost:6379`

### Step 3: Run Database Migrations
Make sure `DATABASE_URL` in `backend/api/.env` is set to your Supabase PostgreSQL database connection, then apply SQL files in order:
```sql
-- Apply db/migrations/*.sql on your Supabase Postgres Editor or via CLI
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

