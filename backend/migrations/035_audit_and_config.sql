-- 配置文件历史（每次写入自动备份）
CREATE TABLE IF NOT EXISTS config_file_history (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    file_path VARCHAR(512) NOT NULL,
    content TEXT NOT NULL,
    admin_user VARCHAR(128) NOT NULL DEFAULT '',
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cfg_history_server_path ON config_file_history(server_id, file_path);
CREATE INDEX IF NOT EXISTS idx_cfg_history_created ON config_file_history(created_at);

-- 审计日志统计视图（物化）
CREATE TABLE IF NOT EXISTS audit_stats_daily (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    stat_date DATE NOT NULL,
    rcon_commands INTEGER NOT NULL DEFAULT 0,
    admin_actions INTEGER NOT NULL DEFAULT 0,
    chat_violations INTEGER NOT NULL DEFAULT 0,
    player_kicks INTEGER NOT NULL DEFAULT 0,
    player_bans INTEGER NOT NULL DEFAULT 0,
    unique_admins INTEGER NOT NULL DEFAULT 0,
    UNIQUE(server_id, stat_date)
);
