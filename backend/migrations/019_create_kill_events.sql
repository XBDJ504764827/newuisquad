CREATE TABLE IF NOT EXISTS kill_events (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    attacker_name VARCHAR(128) NOT NULL DEFAULT '',
    attacker_eos VARCHAR(64) NOT NULL DEFAULT '',
    attacker_steam64 VARCHAR(32) NOT NULL DEFAULT '',
    victim_name VARCHAR(128) NOT NULL DEFAULT '',
    damage DOUBLE PRECISION NOT NULL DEFAULT 0,
    weapon VARCHAR(64) NOT NULL DEFAULT 'Unknown',
    is_kill BOOLEAN NOT NULL DEFAULT false,
    is_teamkill BOOLEAN NOT NULL DEFAULT false,
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_kill_events_server ON kill_events(server_id);
CREATE INDEX IF NOT EXISTS idx_kill_events_time ON kill_events(logged_at DESC);
CREATE INDEX IF NOT EXISTS idx_kill_events_attacker ON kill_events(attacker_steam64);
