CREATE TABLE IF NOT EXISTS team_switch_config (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL UNIQUE REFERENCES servers(id),
    enabled BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_team_switch_config_server ON team_switch_config(server_id);
