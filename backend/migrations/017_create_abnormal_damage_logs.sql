CREATE TABLE IF NOT EXISTS abnormal_damage_logs (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    player_name VARCHAR(128) NOT NULL,
    player_steamid64 VARCHAR(32) NOT NULL,
    victim_name VARCHAR(128) NOT NULL,
    victim_steamid64 VARCHAR(32) NOT NULL,
    weapon VARCHAR(64) NOT NULL,
    damage INTEGER NOT NULL,
    attacker_faction VARCHAR(32) NOT NULL DEFAULT '',
    victim_faction VARCHAR(32) NOT NULL DEFAULT '',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_abnormal_damage_logs_server ON abnormal_damage_logs(server_id);
CREATE INDEX IF NOT EXISTS idx_abnormal_damage_logs_time ON abnormal_damage_logs(logged_at DESC);
CREATE INDEX IF NOT EXISTS idx_abnormal_damage_logs_player ON abnormal_damage_logs(player_steamid64);
