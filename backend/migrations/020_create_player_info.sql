CREATE TABLE IF NOT EXISTS player_info (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    steam64 VARCHAR(32) NOT NULL DEFAULT '',
    eos_id VARCHAR(64) NOT NULL DEFAULT '',
    ip VARCHAR(45) NOT NULL DEFAULT '',
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_player_info_server ON player_info(server_id);
CREATE INDEX IF NOT EXISTS idx_player_info_steam64 ON player_info(steam64);
CREATE INDEX IF NOT EXISTS idx_player_info_name ON player_info(player_name);
CREATE UNIQUE INDEX IF NOT EXISTS idx_player_info_unique ON player_info(server_id, steam64);
