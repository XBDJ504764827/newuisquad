CREATE TABLE IF NOT EXISTS connection_events (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    steam64 VARCHAR(32) NOT NULL DEFAULT '',
    action VARCHAR(16) NOT NULL DEFAULT 'connected',
    ip_address VARCHAR(45) NOT NULL DEFAULT '',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_connection_events_server ON connection_events(server_id);
CREATE INDEX IF NOT EXISTS idx_connection_events_logged ON connection_events(server_id, logged_at DESC);
