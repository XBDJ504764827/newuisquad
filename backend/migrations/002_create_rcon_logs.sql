CREATE TABLE IF NOT EXISTS rcon_logs (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id),
    admin_user VARCHAR(64) NOT NULL,
    command TEXT NOT NULL,
    response TEXT,
    ip_address VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
