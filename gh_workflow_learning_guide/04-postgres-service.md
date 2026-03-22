# PostgreSQL Service Containers

## Why Do We Need PostgreSQL in CI?

This project uses **SQLx** with compile-time query checking. SQLx validates SQL queries against a real database schema at build time. Without a running PostgreSQL instance, the project won't compile.

## Service Container Configuration

```yaml
services:
  postgres:
    image: postgres:16-alpine
    env:
      POSTGRES_USER: metamcp
      POSTGRES_PASSWORD: metamcp_ci_password
      POSTGRES_DB: metamcp_ci
    ports:
      - 5432:5432
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
```

### Breaking it down

| Field | Value | Purpose |
|-------|-------|---------|
| `image` | `postgres:16-alpine` | Lightweight PostgreSQL 16 image (~80MB vs ~400MB for full image) |
| `POSTGRES_USER` | `metamcp` | Database superuser name |
| `POSTGRES_PASSWORD` | `metamcp_ci_password` | Password (CI-only, not sensitive) |
| `POSTGRES_DB` | `metamcp_ci` | Database name created on startup |
| `ports` | `5432:5432` | Maps container port to host (so `localhost:5432` works) |

### Health Check

```yaml
options: >-
  --health-cmd pg_isready
  --health-interval 10s
  --health-timeout 5s
  --health-retries 5
```

GitHub Actions waits for the service to be **healthy** before running job steps. The health check:

1. Runs `pg_isready` every 10 seconds
2. Allows 5 seconds per check
3. Retries up to 5 times
4. Total maximum wait: ~75 seconds

Without this, steps might fail because PostgreSQL isn't ready yet.

## The Migration Step

```yaml
- name: Install SQLx CLI
  run: cargo install sqlx-cli --no-default-features --features rustls,postgres
- name: Run migrations
  run: sqlx migrate run
```

After PostgreSQL is healthy:

1. **Install SQLx CLI** — the `sqlx` command-line tool for managing migrations
   - `--no-default-features` — disables SQLite and MySQL support (faster install)
   - `--features rustls,postgres` — only PostgreSQL with Rust-native TLS
2. **Run migrations** — applies all SQL files from `migrations/` to create tables

The migrations create the schema that SQLx validates queries against at compile time.

## Connection String

```yaml
env:
  DATABASE_URL: postgresql://metamcp:metamcp_ci_password@localhost:5432/metamcp_ci
```

This matches the service container's credentials exactly:
- User: `metamcp`
- Password: `metamcp_ci_password`
- Host: `localhost` (port-mapped from the container)
- Port: `5432`
- Database: `metamcp_ci`

## Which Jobs Need PostgreSQL?

| Job | Needs PostgreSQL? | Why? |
|-----|-------------------|------|
| `fmt` | No | Only checks formatting syntax |
| `clippy` | Yes | Clippy compiles code, triggering SQLx checks |
| `build` | Yes | Building compiles code, triggering SQLx checks |
| `test` | Yes | Tests run queries against the database |

## Alternative: SQLx Offline Mode

Instead of running PostgreSQL in CI, you can use SQLx offline mode:

```bash
# Locally: generate query metadata
cargo sqlx prepare

# This creates a .sqlx/ directory with cached query information
# Commit this directory to git
```

Then in CI, set `SQLX_OFFLINE=true` and you won't need a database. However, this project uses the live-database approach for stronger guarantees.
