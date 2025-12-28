# MetaMCP Development Setup

Quick setup guide for local development.

## Prerequisites

- **Rust** (latest stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Docker & Docker Compose**: [Install Docker](https://docs.docker.com/get-docker/)
- **PostgreSQL Client** (optional): For manual database access

## Quick Start

### 1. Clone and Setup Environment

```bash
cd metamcp_rust

# Copy environment template
cp .env.example .env

# Generate secrets
JWT_SECRET=$(openssl rand -base64 32)
ENCRYPTION_KEY=$(openssl rand -hex 32)

# Update .env file with generated secrets
# On macOS/Linux:
sed -i.bak "s|JWT_SECRET=.*|JWT_SECRET=${JWT_SECRET}|" .env
sed -i.bak "s|ENCRYPTION_KEY=.*|ENCRYPTION_KEY=${ENCRYPTION_KEY}|" .env

# Or manually edit .env file with your preferred editor
nano .env  # or vim, code, etc.
```

**Important:** The `.env` file contains database credentials and secrets. It is used by:
- Docker Compose (for PostgreSQL container configuration)
- SQLx CLI (for running migrations)
- MetaMCP server (for database connections)

### 2. Configure Database (Optional)

The default `.env` contains development defaults. To customize:

```bash
# Edit .env and modify:
POSTGRES_USER=your_username        # Default: metamcp
POSTGRES_PASSWORD=your_password    # Default: metamcp_dev_password
POSTGRES_DB=your_database_name     # Default: metamcp_dev
POSTGRES_PORT=5432                 # Default: 5432

# Update DATABASE_URL to match:
DATABASE_URL=postgresql://your_username:your_password@localhost:5432/your_database_name
```

### 3. Start PostgreSQL

```bash
# Start PostgreSQL in Docker (reads credentials from .env)
docker-compose up -d postgres

# Verify it's running
docker-compose ps

# Check logs
docker-compose logs -f postgres

# Test connection using credentials from .env
source .env
docker exec -it metamcp-postgres psql -U $POSTGRES_USER -d $POSTGRES_DB
```

### 4. Set Up Database Schema

```bash
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations (when they exist)
# The DATABASE_URL from .env is automatically used
sqlx migrate run

# Verify database connection
source .env
docker exec -it metamcp-postgres psql -U $POSTGRES_USER -d $POSTGRES_DB
```

### 5. Build and Run

```bash
# Build the project
cargo build

# Run the server (when implemented)
cargo run --bin metamcp

# Run the CLI tool (when implemented)
cargo run --bin metamcp-cli -- keys list
```

## Development Workflow

### Managing Database

```bash
# View PostgreSQL logs
docker-compose logs -f postgres

# Connect to database
docker exec -it metamcp-postgres psql -U metamcp -d metamcp_dev

# Stop database
docker-compose down

# Reset database (WARNING: deletes all data)
docker-compose down -v
docker-compose up -d postgres
sqlx migrate run
```

### Creating Migrations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run all pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

### Using the CLI (After Phase 3)

```bash
# Create an API key
cargo run --bin metamcp-cli keys create --name "Development Key"

# List all keys
cargo run --bin metamcp-cli keys list

# Show key details
cargo run --bin metamcp-cli keys show <key-id>

# Inactivate a key
cargo run --bin metamcp-cli keys inactivate <key-id>

# Delete a key
cargo run --bin metamcp-cli keys delete <key-id> --confirm
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'

# Run with logging
RUST_LOG=debug cargo test
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy

# Run linter with all warnings
cargo clippy -- -W clippy::all

# Check compilation without building
cargo check
```

## Useful Commands

### Docker Commands

```bash
# View all containers
docker-compose ps

# View resource usage
docker stats metamcp-postgres

# Execute psql in container
docker exec -it metamcp-postgres psql -U metamcp -d metamcp_dev

# Backup database
docker exec metamcp-postgres pg_dump -U metamcp metamcp_dev > backup.sql

# Restore database
docker exec -i metamcp-postgres psql -U metamcp metamcp_dev < backup.sql
```

### PostgreSQL Commands (inside psql)

```sql
-- List databases
\l

-- Connect to database
\c metamcp_dev

-- List tables
\dt

-- Describe table
\d api_keys

-- Show table contents
SELECT * FROM api_keys;

-- Quit
\q
```

## Troubleshooting

### Port 5432 Already in Use

```bash
# Check what's using the port
lsof -i :5432

# Stop local PostgreSQL
brew services stop postgresql  # macOS
sudo systemctl stop postgresql # Linux

# Or use different port in docker-compose.yml
```

### Database Connection Issues

```bash
# Check container is running
docker ps | grep metamcp-postgres

# Check logs
docker logs metamcp-postgres

# Restart container
docker-compose restart postgres

# Verify connection
docker exec metamcp-postgres pg_isready -U metamcp -d metamcp_dev
```

### SQLx Compilation Errors

```bash
# Make sure DATABASE_URL is set
export DATABASE_URL=postgresql://metamcp:metamcp_dev_password@localhost:5432/metamcp_dev

# Prepare SQLx metadata for offline builds
cargo sqlx prepare

# Check database connection
cargo sqlx database create
cargo sqlx migrate run
```

### Clean Rebuild

```bash
# Clean build artifacts
cargo clean

# Remove Docker containers and volumes
docker-compose down -v

# Start fresh
docker-compose up -d postgres
sqlx migrate run
cargo build
```

## Environment Variables

### How .env Works

The `.env` file is automatically read by:
- **Docker Compose**: Uses variables for PostgreSQL container configuration
- **Rust application**: Loads via `dotenv` crate at runtime
- **SQLx CLI**: Reads `DATABASE_URL` for migrations

### Key Variables

See `.env.example` for all available configuration options.

**Database Configuration:**
- `POSTGRES_USER`: Database username (used by Docker & app)
- `POSTGRES_PASSWORD`: Database password (used by Docker & app)
- `POSTGRES_DB`: Database name (used by Docker & app)
- `POSTGRES_HOST`: Database host (default: localhost)
- `POSTGRES_PORT`: Database port (default: 5432)
- `DATABASE_URL`: Full connection string (must match above values)

**Security:**
- `JWT_SECRET`: Secret for signing JWT tokens (generate with `openssl rand -base64 32`)
- `ENCRYPTION_KEY`: Key for encrypting API keys at rest (generate with `openssl rand -hex 32`)

**Server:**
- `SERVER_HOST`: Server bind address (default: 127.0.0.1)
- `SERVER_PORT`: Server port (default: 12009)

**Logging:**
- `RUST_LOG`: Logging level (options: error, warn, info, debug, trace)

### Updating Credentials

When changing database credentials in `.env`:

1. Update all related variables:
   ```bash
   POSTGRES_USER=new_user
   POSTGRES_PASSWORD=new_password
   POSTGRES_DB=new_db
   DATABASE_URL=postgresql://new_user:new_password@localhost:5432/new_db
   ```

2. Restart Docker container:
   ```bash
   docker-compose down -v  # Warning: removes all data
   docker-compose up -d postgres
   ```

3. Run migrations:
   ```bash
   sqlx migrate run
   ```

## Next Steps

1. Review [MIGRATION_PLAN.md](./MIGRATION_PLAN.md) for architecture details
2. Start with Phase 1: Core Infrastructure
3. Follow the migration phases sequentially
4. Use the CLI tool for API key management
5. Build test components for validation

## Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Tokio Documentation](https://tokio.rs/)
- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
