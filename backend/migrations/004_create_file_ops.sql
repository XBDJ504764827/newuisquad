CREATE TABLE IF NOT EXISTS file_ops (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id),
    admin_user VARCHAR(64) NOT NULL,
    operation VARCHAR(16) NOT NULL,
    file_path TEXT NOT NULL,
    content TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
