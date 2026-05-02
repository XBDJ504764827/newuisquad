CREATE TABLE IF NOT EXISTS chat_messages (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    steam64 VARCHAR(32) NOT NULL DEFAULT '',
    message TEXT NOT NULL DEFAULT '',
    channel VARCHAR(32) NOT NULL DEFAULT 'All',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_chat_messages_server ON chat_messages(server_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_time ON chat_messages(logged_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_messages_player ON chat_messages(steam64);
