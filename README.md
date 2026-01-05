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

## End-to-End Usage with Claude CLI

MetaMCP acts as a gateway that aggregates multiple MCP servers, allowing Claude CLI to access all tools through a single endpoint.

### Step 1: Start the Services

```bash
# Terminal 1: Start PostgreSQL
docker-compose up -d postgres

# Terminal 2: Start MetaMCP server
cargo run --bin metamcp

# Terminal 3: Start example backend server 1 (simple tools)
cargo run --example backend_server_1 -- --port 3001

# Terminal 4: Start example backend server 2 (advanced tools + resources + prompts)
cargo run --example backend_server_2 -- --port 3002
```

### Step 2: Create API Key and Register MCP Servers

```bash
# Create an API key
cargo run --bin metamcp-cli -- keys create --name "claude-cli"
# Save the API key output (e.g., mcp_xxx...)

# Get JWT token (note: response field is "access_token", not "token")
TOKEN=$(curl -s -X POST http://localhost:12009/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"api_key": "mcp_xxx..."}' | jq -r '.access_token')

# Register backend server 1 (simple tools: echo, add, uppercase, reverse, timestamp)
curl -X POST http://localhost:12009/api/v1/mcp/servers \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "simple-tools", "url": "http://localhost:3001", "protocol": "http"}'

# Register backend server 2 (advanced: file ops, base64, prompts, resources)
curl -X POST http://localhost:12009/api/v1/mcp/servers \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "advanced-tools", "url": "http://localhost:3002", "protocol": "http"}'
```

### Step 3: Configure Claude CLI

Add MetaMCP to your Claude CLI configuration (`~/.claude/claude_desktop_config.json` or similar):

```json
{
  "mcpServers": {
    "metamcp": {
      "url": "http://localhost:12009/mcp",
      "headers": {
        "Authorization": "Bearer <your-jwt-token>"
      }
    }
  }
}
```

Or using API key authentication:

```json
{
  "mcpServers": {
    "metamcp": {
      "url": "http://localhost:12009/mcp",
      "headers": {
        "X-API-Key": "mcp_xxx..."
      }
    }
  }
}
```

### Step 4: Test MCP Gateway with curl

Before using Claude CLI, you can verify the MCP gateway is working:

```bash
# List all available tools (aggregated from all backend servers)
curl -X POST http://localhost:12009/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

# Call the 'add' tool from simple-tools server
curl -X POST http://localhost:12009/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"simple-tools_add","arguments":{"a":5,"b":3}}}'

# Call the 'echo' tool
curl -X POST http://localhost:12009/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"simple-tools_echo","arguments":{"message":"Hello from MetaMCP!"}}}'

# Call the 'base64_encode' tool from advanced-tools server
curl -X POST http://localhost:12009/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"advanced-tools_base64_encode","arguments":{"text":"Hello World"}}}'
```

**Note:** Tool names are prefixed with the server name (e.g., `simple-tools_add`) to avoid collisions between servers.

### Step 5: Use with Claude CLI

Once configured, Claude CLI will have access to all tools from registered MCP servers:

**From Backend Server 1:**
- `echo` - Echoes back input message
- `add` - Adds two numbers
- `uppercase` - Converts text to uppercase
- `reverse` - Reverses a string
- `timestamp` - Returns current Unix timestamp

**From Backend Server 2:**
- `read_file` - Reads from virtual file system
- `write_file` - Writes to virtual file system
- `list_files` - Lists all virtual files
- `parse_json` - Parses and formats JSON
- `base64_encode` / `base64_decode` - Base64 encoding/decoding
- `word_count` - Counts words, characters, lines
- `get_config` - Gets configuration values

**Resources available:**
- `config://server` - Server configuration
- `config://all` - All configuration values
- `file://readme.txt`, `file://config.json`, `file://notes.md` - Virtual files

**Prompts available:**
- `code_review` - Generate code review feedback
- `summarize` - Summarize text
- `explain` - Explain concepts

### Example Session

```bash
# Start Claude CLI with MetaMCP configured
claude

# Claude can now use aggregated tools:
# "Use the add tool to calculate 42 + 17"
# "Read the file readme.txt from the virtual file system"
# "Encode 'Hello World' to base64"
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
