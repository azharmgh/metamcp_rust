-- Create API keys table
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    encrypted_key BYTEA NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,

    CONSTRAINT api_keys_name_check CHECK (char_length(name) > 0)
);

-- Index for listing active keys
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(is_active, created_at DESC);
