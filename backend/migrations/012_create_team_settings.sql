CREATE TABLE IF NOT EXISTS team_settings (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL UNIQUE REFERENCES servers(id),
    create_team_broadcast BOOLEAN NOT NULL DEFAULT true,
    captain_time_check BOOLEAN NOT NULL DEFAULT false,
    captain_min_playtime INTEGER NOT NULL DEFAULT 30,
    captain_check_min_players INTEGER NOT NULL DEFAULT 20,
    max_create_team_attempts INTEGER NOT NULL DEFAULT 3,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
