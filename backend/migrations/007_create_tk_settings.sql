CREATE TABLE IF NOT EXISTS tk_settings (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL UNIQUE REFERENCES servers(id),
    enabled BOOLEAN NOT NULL DEFAULT false,
    max_team_kills INTEGER NOT NULL DEFAULT 3,
    apology_time_minutes INTEGER NOT NULL DEFAULT 5,
    notification_message TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
