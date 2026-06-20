# Local Development Runbook

This guide covers the manual process to run the fullstack WhatsUp platform locally, bypassing the backend Docker builds for faster iteration and to avoid network timeouts.

## Prerequisites
- Docker & Docker Compose
- Rust & Cargo
- Node.js & npm

## 1. Start the Infrastructure (Terminal 1)
Instead of running `docker compose up -d` (which attempts to build the frontend and backend), we only want to start the database and storage services.

```powershell
# Open a terminal in the root project folder
cd C:\Users\DELL\IdeaProjects\ETE-WHATSUP

# Start Postgres (Port 5434), Redis, MinIO, and n8n
docker compose up -d postgres redis minio n8n
```

## 2. Start the Backend API & Workers (Terminal 2)
The backend API server and background workers are bundled into a single Rust binary (`whatsup-api`).

```powershell
# Open a second terminal
cd C:\Users\DELL\IdeaProjects\ETE-WHATSUP\backend\api

# Run the API locally (defaults to port 8081)
cargo run --bin whatsup-api
```
*Note: If you get an `os error 10048`, port 8081 is already in use by a previous crashed instance. Find the process using `netstat -ano | findstr :8081` and kill it using `taskkill /PID <PID> /F`.*

## 3. Start the Frontend (Terminal 3)
The Next.js frontend connects to the backend running on `localhost:8081`.

```powershell
# Open a third terminal
cd C:\Users\DELL\IdeaProjects\ETE-WHATSUP\frontend

# Start the development server
npm run dev
```

## 4. Access the Platform
- **Frontend App:** http://localhost:3000
- **MinIO Console:** http://localhost:9001
- **n8n Automation:** http://localhost:5678

## Troubleshooting
- **Postgres Authentication Failed:** Ensure you don't have another local Postgres instance stealing the port. Our `docker-compose.yml` maps to port `5434` to avoid this.
- **Frontend Hydration Error:** If you see `data-qb-installed`, it is caused by a browser extension (like Quillbot) injecting tags into the DOM. This warning is suppressed in `layout.tsx` and can be ignored.
- **Can't Login after Registering:** Make sure your backend logic bypasses email verification during development, otherwise newly registered accounts get stuck in a `pending_verification` state.
