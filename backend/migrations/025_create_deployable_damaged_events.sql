CREATE TABLE IF NOT EXISTS deployable_damaged_events (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    deployable VARCHAR(128) NOT NULL DEFAULT '',
    damage DOUBLE PRECISION NOT NULL DEFAULT 0,
    weapon VARCHAR(128) NOT NULL DEFAULT '',
    player_suffix VARCHAR(128) NOT NULL DEFAULT '',
    damage_type VARCHAR(64) NOT NULL DEFAULT '',
    health_remaining DOUBLE PRECISION NOT NULL DEFAULT 0,
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_deployable_damaged_server ON deployable_damaged_events(server_id);
CREATE INDEX IF NOT EXISTS idx_deployable_damaged_time ON deployable_damaged_events(logged_at DESC);
