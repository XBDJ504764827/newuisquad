CREATE TABLE IF NOT EXISTS server_logs (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id),
    log_level VARCHAR(16) NOT NULL,
    category VARCHAR(32),
    message TEXT NOT NULL,
    raw_line TEXT,
    logged_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_server_logs_server_id ON server_logs(server_id);
CREATE INDEX IF NOT EXISTS idx_server_logs_logged_at ON server_logs(logged_at DESC);
CREATE INDEX IF NOT EXISTS idx_server_logs_level ON server_logs(log_level);
