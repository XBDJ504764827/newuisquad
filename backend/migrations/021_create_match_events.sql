CREATE TABLE IF NOT EXISTS team_assignments (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    steam64 VARCHAR(32) NOT NULL DEFAULT '',
    team_id INTEGER NOT NULL DEFAULT 0,
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS squad_creations (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    steam64 VARCHAR(32) NOT NULL DEFAULT '',
    squad_id VARCHAR(16) NOT NULL DEFAULT '',
    squad_name VARCHAR(128) NOT NULL DEFAULT '',
    faction VARCHAR(64) NOT NULL DEFAULT '',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS match_info (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    map_name VARCHAR(128) NOT NULL DEFAULT '',
    layer_name VARCHAR(128) NOT NULL DEFAULT '',
    team1_faction VARCHAR(64) NOT NULL DEFAULT '',
    team2_faction VARCHAR(64) NOT NULL DEFAULT '',
    winner_team INTEGER,
    event_type VARCHAR(32) NOT NULL DEFAULT 'layer_change',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS revive_events (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    reviver_name VARCHAR(128) NOT NULL DEFAULT '',
    reviver_steam64 VARCHAR(32) NOT NULL DEFAULT '',
    revived_name VARCHAR(128) NOT NULL DEFAULT '',
    revived_steam64 VARCHAR(32) NOT NULL DEFAULT '',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS vehicle_events (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    steam64 VARCHAR(32) NOT NULL DEFAULT '',
    vehicle_name VARCHAR(128) NOT NULL DEFAULT '',
    event_type VARCHAR(16) NOT NULL DEFAULT 'enter',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS admin_actions (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    admin_name VARCHAR(128) NOT NULL DEFAULT '',
    action_type VARCHAR(64) NOT NULL DEFAULT '',
    target VARCHAR(256) NOT NULL DEFAULT '',
    message TEXT NOT NULL DEFAULT '',
    raw_line TEXT NOT NULL DEFAULT '',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_team_assign_server ON team_assignments(server_id);
CREATE INDEX IF NOT EXISTS idx_squad_create_server ON squad_creations(server_id);
CREATE INDEX IF NOT EXISTS idx_match_info_server ON match_info(server_id);
CREATE INDEX IF NOT EXISTS idx_admin_actions_server ON admin_actions(server_id);
