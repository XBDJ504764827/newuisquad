CREATE TABLE IF NOT EXISTS damage_notify_settings (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id),
    enabled BOOLEAN NOT NULL DEFAULT false,
    keyword VARCHAR(64) NOT NULL DEFAULT '!damage',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
