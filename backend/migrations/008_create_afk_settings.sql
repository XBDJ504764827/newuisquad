CREATE TABLE IF NOT EXISTS afk_settings (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL UNIQUE REFERENCES servers(id),
    enabled BOOLEAN NOT NULL DEFAULT false,
    min_players_to_check INTEGER NOT NULL DEFAULT 10,
    max_afk_minutes INTEGER NOT NULL DEFAULT 15,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
