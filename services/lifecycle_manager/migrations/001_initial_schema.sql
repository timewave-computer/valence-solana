-- Initial schema for Valence Lifecycle Manager

-- Account requests table
CREATE TABLE IF NOT EXISTS account_requests (
    id VARCHAR(44) PRIMARY KEY,
    owner VARCHAR(44) NOT NULL,
    capabilities TEXT[] NOT NULL,
    init_state_hash BYTEA NOT NULL,
    created_at BIGINT NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    account_id VARCHAR(44),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_account_requests_status ON account_requests(status);
CREATE INDEX idx_account_requests_created_at ON account_requests(created_at);

-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id VARCHAR(44) PRIMARY KEY,
    owner VARCHAR(44) NOT NULL,
    capabilities TEXT[] NOT NULL,
    state_hash BYTEA NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at BIGINT NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_accounts_owner ON accounts(owner);
CREATE INDEX idx_accounts_active ON accounts(is_active);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(44) PRIMARY KEY,
    owner VARCHAR(44) NOT NULL,
    accounts TEXT[] NOT NULL,
    namespace VARCHAR(64) NOT NULL,
    is_consumed BOOLEAN DEFAULT false,
    nonce BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    metadata BYTEA,
    consumed_at BIGINT,
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_sessions_owner ON sessions(owner);
CREATE INDEX idx_sessions_consumed ON sessions(is_consumed);
CREATE INDEX idx_sessions_namespace ON sessions(namespace);

-- Session consumptions table
CREATE TABLE IF NOT EXISTS session_consumptions (
    id SERIAL PRIMARY KEY,
    consumed_session VARCHAR(44) NOT NULL,
    created_sessions TEXT[] NOT NULL,
    transaction_signature BYTEA NOT NULL,
    consumed_at BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_consumptions_consumed_session ON session_consumptions(consumed_session);

-- Linear progressions table
CREATE TABLE IF NOT EXISTS linear_progressions (
    id VARCHAR(100) PRIMARY KEY,
    current_state JSONB NOT NULL,
    history JSONB NOT NULL DEFAULT '[]',
    pending_operations JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_progressions_updated_at ON linear_progressions(updated_at);

-- Lifecycle events table
CREATE TABLE IF NOT EXISTS lifecycle_events (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_events_type ON lifecycle_events(event_type);
CREATE INDEX idx_events_created_at ON lifecycle_events(created_at);

-- Progression rules table
CREATE TABLE IF NOT EXISTS progression_rules (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    condition JSONB NOT NULL,
    action JSONB NOT NULL,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_rules_enabled ON progression_rules(enabled);

-- Update triggers
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_account_requests_updated_at BEFORE UPDATE ON account_requests
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_accounts_updated_at BEFORE UPDATE ON accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sessions_updated_at BEFORE UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_progressions_updated_at BEFORE UPDATE ON linear_progressions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rules_updated_at BEFORE UPDATE ON progression_rules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();