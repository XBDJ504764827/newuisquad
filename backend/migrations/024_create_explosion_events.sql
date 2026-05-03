CREATE TABLE IF NOT EXISTS explosion_events (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    pos_x REAL NOT NULL DEFAULT 0,
    pos_y REAL NOT NULL DEFAULT 0,
    pos_z REAL NOT NULL DEFAULT 0,
    damage_causer VARCHAR(128) NOT NULL DEFAULT '',
    damage_instigator VARCHAR(128) NOT NULL DEFAULT '',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_explosion_server ON explosion_events(server_id);
CREATE INDEX IF NOT EXISTS idx_explosion_time ON explosion_events(logged_at DESC);
