-- 聊天审核设置
CREATE TABLE IF NOT EXISTS chat_moderation_settings (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT false,
    -- 内置过滤器
    enable_racial_slurs BOOLEAN NOT NULL DEFAULT true,
    enable_homophobic_slurs BOOLEAN NOT NULL DEFAULT true,
    enable_ableist_language BOOLEAN NOT NULL DEFAULT true,
    enable_chinese_slurs BOOLEAN NOT NULL DEFAULT true,
    -- 自定义
    custom_blacklist TEXT[] NOT NULL DEFAULT '{}',
    whitelist TEXT[] NOT NULL DEFAULT '{}',
    -- 升级处罚（JSON数组）
    escalation_actions JSONB NOT NULL DEFAULT '[
        {"violation_count": 1, "action": "WARN", "message": "请注意文明用语"},
        {"violation_count": 2, "action": "WARN", "message": "再次警告，继续违规将被踢出"},
        {"violation_count": 3, "action": "KICK", "message": "因多次违规聊天被踢出"},
        {"violation_count": 4, "action": "BAN", "ban_duration_days": 1, "message": "因多次违规聊天被封禁1天"}
    ]'::jsonb,
    -- 其他
    violation_expiry_days INTEGER NOT NULL DEFAULT 30,
    exempt_admins BOOLEAN NOT NULL DEFAULT true,
    log_detections BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(server_id)
);

-- 聊天违规记录
CREATE TABLE IF NOT EXISTS chat_violations (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    steam_id VARCHAR(32) NOT NULL,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    message TEXT NOT NULL DEFAULT '',
    category VARCHAR(32) NOT NULL DEFAULT '',
    matched_word VARCHAR(64) NOT NULL DEFAULT '',
    action_taken VARCHAR(16) NOT NULL DEFAULT '',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chat_violations_server_steam ON chat_violations(server_id, steam_id);
CREATE INDEX IF NOT EXISTS idx_chat_violations_logged ON chat_violations(logged_at);
