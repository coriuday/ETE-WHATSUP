# Migrations

SQL files in `db/migrations/` are applied **externally** (not on API boot).

Apply in lexical order:

1. `001`–`015` original schema
2. `016_provider_lifecycle.sql` — provider column, extra message statuses, status events, quick replies
3. `017_automations.sql` — native workflows and runs

Example:

```bash
for f in db/migrations/*.sql; do psql "$DATABASE_URL" -f "$f"; done
```

Compose Postgres typically listens on host port **5434**.
