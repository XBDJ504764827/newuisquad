CREATE TABLE IF NOT EXISTS system_logs (
    id SERIAL PRIMARY KEY,
    log_type VARCHAR(32) NOT NULL DEFAULT 'backend',
    level VARCHAR(16) NOT NULL DEFAULT 'INFO',
    module VARCHAR(64) NOT NULL DEFAULT '',
    message TEXT NOT NULL DEFAULT '',
    detail TEXT NOT NULL DEFAULT '',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_system_logs_type ON system_logs(log_type);
CREATE INDEX IF NOT EXISTS idx_system_logs_time ON system_logs(logged_at DESC);
