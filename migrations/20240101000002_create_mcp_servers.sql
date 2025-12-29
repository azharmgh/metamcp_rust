-- Create MCP servers configuration table
CREATE TABLE IF NOT EXISTS mcp_servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    url VARCHAR(512) NOT NULL,
    protocol VARCHAR(50) NOT NULL DEFAULT 'http',
    command TEXT,
    args JSONB,
    env JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT mcp_servers_name_check CHECK (char_length(name) > 0),
    CONSTRAINT mcp_servers_protocol_check CHECK (protocol IN ('http', 'sse', 'stdio'))
);

-- Index for listing active servers
CREATE INDEX IF NOT EXISTS idx_mcp_servers_active ON mcp_servers(is_active, name);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

DROP TRIGGER IF EXISTS update_mcp_servers_updated_at ON mcp_servers;
CREATE TRIGGER update_mcp_servers_updated_at
    BEFORE UPDATE ON mcp_servers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
