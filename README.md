# MetaMCP - Rust Backend

This is an experimental repo to re-write the backend of [metamcp by metatool-ai](https://github.com/metatool-ai/metamcp/tree/main/apps/backend) from JavaScript to Rust.

This repo and the code is created with Anthropic Claude Code AI assistant.

## Features

- 🦀 **Pure Rust** - High performance, type-safe backend
- 🔐 **API Key Authentication** - Simple, secure authentication without user management
- 📡 **Streaming HTTP** - MCP protocol over HTTP with streaming support
- 🐘 **PostgreSQL** - Reliable data persistence with SQLx
- 🔧 **CLI Tool** - Dedicated CLI for API key management
- 🚀 **Docker Ready** - Easy development setup with Docker Compose

## Quick Start

### 1. Setup Environment

```bash
# Copy environment template
cp .env.example .env

# Generate secrets and update .env
JWT_SECRET=$(openssl rand -base64 32)
ENCRYPTION_KEY=$(openssl rand -hex 32)
sed -i.bak "s|JWT_SECRET=.*|JWT_SECRET=${JWT_SECRET}|" .env
sed -i.bak "s|ENCRYPTION_KEY=.*|ENCRYPTION_KEY=${ENCRYPTION_KEY}|" .env
```

**Important:** All configuration is in `.env` file, used by:
- Docker Compose (PostgreSQL credentials)
- SQLx CLI (database migrations)
- MetaMCP server (runtime configuration)

### 2. Start Database

```bash
# Start PostgreSQL (reads credentials from .env automatically)
docker-compose up -d postgres

# Verify
docker-compose ps
```

### 3. Run Migrations

```bash
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations (uses DATABASE_URL from .env)
sqlx migrate run
```

### 4. Build and Run

```bash
# Build (when implemented)
cargo build

# Run server (when implemented)
cargo run --bin metamcp

# Use CLI tool (when implemented)
cargo run --bin metamcp-cli -- keys list
```

## Documentation

- **[SETUP.md](./SETUP.md)** - Detailed development setup guide
- **[MIGRATION_PLAN.md](./MIGRATION_PLAN.md)** - Complete architecture and migration plan

## Architecture

- **Framework**: Axum (async, type-safe)
- **Database**: PostgreSQL with SQLx (compile-time checked queries)
- **Authentication**: API Key + JWT (no user accounts)
- **Client Protocol**: Streaming HTTP only
- **Backend Servers**: HTTP/SSE/stdio (with protocol translation)

## Project Status

🚧 **In Development** - Following phased migration plan:

- ✅ Planning & Architecture
- ⏳ Phase 1: Core Infrastructure
- ⏳ Phase 2: Database Layer & API Keys
- ⏳ Phase 3: CLI Tool & Authentication
- ⏳ Phase 4: MCP Protocol Implementation

See [MIGRATION_PLAN.md](./MIGRATION_PLAN.md) for details.

## Contributing

This is an experimental project created with AI assistance. See [MIGRATION_PLAN.md](./MIGRATION_PLAN.md) for architecture decisions and implementation guidelines.

## License

[Add license information]

