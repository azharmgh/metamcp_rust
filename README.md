# MetaMCP - Rust Backend

A high-performance Rust implementation of the [MetaMCP backend](https://github.com/metatool-ai/metamcp/tree/main/apps/backend), rewritten from JavaScript for improved performance and type safety.

This repo and the code is created with Anthropic Claude Code AI assistant.

## Features

- 🦀 **Pure Rust** - High performance, type-safe backend with Axum
- 🔐 **API Key Authentication** - Secure API keys with JWT tokens (no user management)
- 📡 **MCP Protocol** - Full Model Context Protocol support (tools, resources, prompts)
- 🐘 **PostgreSQL** - Reliable data persistence with SQLx (compile-time checked)
- 🔧 **CLI Tool** - Dedicated CLI for API key management
- 📖 **OpenAPI/Swagger** - Auto-generated API documentation
- 🧪 **Comprehensive Testing** - Unit tests, integration tests, and benchmarks
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
# Build
cargo build

# Run server
cargo run --bin metamcp

# Server will be available at http://localhost:12009
# Swagger UI at http://localhost:12009/swagger-ui
```

### 5. Create API Key

```bash
# Use CLI to create an API key
cargo run --bin metamcp-cli -- keys create --name "my-key"

# List all keys
cargo run --bin metamcp-cli -- keys list
```

## API Usage

### Authentication

```bash
# Get JWT token from API key
curl -X POST http://localhost:12009/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"api_key": "mcp_your_api_key_here"}'
```

### MCP Server Management

```bash
# List MCP servers
curl http://localhost:12009/api/v1/mcp/servers \
  -H "Authorization: Bearer <jwt_token>"

# Create MCP server
curl -X POST http://localhost:12009/api/v1/mcp/servers \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-server", "url": "http://localhost:3001", "protocol": "http"}'
```

## CLI Commands

```bash
# API Key Management
metamcp-cli keys list [--include-inactive]
metamcp-cli keys create --name <name>
metamcp-cli keys show <key-id>
metamcp-cli keys activate <key-id>
metamcp-cli keys inactivate <key-id>
metamcp-cli keys delete <key-id> --confirm
metamcp-cli keys rotate <key-id>
```

## Testing

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run benchmarks
cargo bench

# Run example backend servers
cargo run --example backend_server_1 -- --port 3001
cargo run --example backend_server_2 -- --port 3002

# Run test client
cargo run --example test_client -- --api-key <API_KEY>
```

## Documentation

- **[SETUP.md](./SETUP.md)** - Detailed development setup guide
- **[MIGRATION_PLAN.md](./MIGRATION_PLAN.md)** - Complete architecture and migration plan
- **[SHUTTLE_DEPLOY.md](./SHUTTLE_DEPLOY.md)** - Shuttle.dev deployment guide

## Architecture

```
metamcp_rust/
├── src/
│   ├── api/          # HTTP handlers and routes
│   ├── auth/         # JWT and API key authentication
│   ├── config/       # Configuration management
│   ├── db/           # Database models and repositories
│   ├── mcp/          # MCP protocol implementation
│   ├── streaming/    # Stream manager for events
│   └── utils/        # Error handling utilities
├── migrations/       # SQLx database migrations
├── examples/         # Test client and backend servers
├── tests/            # Unit and integration tests
└── benches/          # Performance benchmarks
```

### Key Components

- **Framework**: Axum (async, type-safe web framework)
- **Database**: PostgreSQL with SQLx (compile-time checked queries)
- **Authentication**: API Key + JWT (stateless, no sessions)
- **MCP Protocol**: Native Rust implementation (JSON-RPC 2.0)
- **API Docs**: OpenAPI 3.0 with Swagger UI

## Project Status

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1: Core Infrastructure | ✅ Complete | Axum server, config, database pool |
| Phase 2: Database Layer | ✅ Complete | API keys, MCP server schemas |
| Phase 3: Authentication & CLI | ✅ Complete | JWT, middleware, CLI tool |
| Phase 4: MCP Protocol | ✅ Complete | Protocol types, server manager |
| Phase 5: API Endpoints | ✅ Complete | REST API, OpenAPI docs |
| Phase 6: Streaming HTTP | 🔄 Partial | Stream manager implemented |
| Phase 7: Testing | ✅ Complete | Unit, integration, benchmarks |
| Phase 8: Optimization | ⬜ Planned | Performance tuning |
| Phase 9: Future | ⬜ Planned | SSE/stdio support |

See [MIGRATION_PLAN.md](./MIGRATION_PLAN.md) for detailed implementation status.

## Contributing

This is an experimental project created with AI assistance. See [MIGRATION_PLAN.md](./MIGRATION_PLAN.md) for architecture decisions and implementation guidelines.

## License

MIT License - See [LICENSE](./LICENSE) for details.
