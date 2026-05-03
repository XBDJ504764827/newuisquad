CREATE TABLE IF NOT EXISTS tick_rate_events (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    tick_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_tick_rate_server ON tick_rate_events(server_id);
CREATE INDEX IF NOT EXISTS idx_tick_rate_time ON tick_rate_events(logged_at DESC);
