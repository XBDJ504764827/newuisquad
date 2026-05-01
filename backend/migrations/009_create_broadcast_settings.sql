CREATE TABLE IF NOT EXISTS broadcast_settings (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL UNIQUE REFERENCES servers(id),
    join_message_enabled BOOLEAN NOT NULL DEFAULT true,
    join_message TEXT NOT NULL DEFAULT '欢迎 {player} 加入服务器',
    gameop_list_enabled BOOLEAN NOT NULL DEFAULT false,
    gameop_list_message TEXT NOT NULL DEFAULT '在线管理员: {oplist}',
    announcement_enabled BOOLEAN NOT NULL DEFAULT false,
    announcement_content TEXT,
    announcement_interval INTEGER NOT NULL DEFAULT 10,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
