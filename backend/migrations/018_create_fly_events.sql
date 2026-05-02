CREATE TABLE IF NOT EXISTS fly_events (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    eos_id VARCHAR(64) NOT NULL DEFAULT '',
    steam64 VARCHAR(32) NOT NULL DEFAULT '',
    event_type VARCHAR(32) NOT NULL DEFAULT 'possess',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_fly_events_server ON fly_events(server_id);
CREATE INDEX IF NOT EXISTS idx_fly_events_time ON fly_events(logged_at DESC);
CREATE INDEX IF NOT EXISTS idx_fly_events_player ON fly_events(steam64);
