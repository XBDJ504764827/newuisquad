-- 权限组定义
CREATE TABLE IF NOT EXISTS permission_groups (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    group_name VARCHAR(64) NOT NULL,
    permissions TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(server_id, group_name)
);

-- 管理员 SteamID → 权限组 映射
CREATE TABLE IF NOT EXISTS permission_admins (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    steam_id VARCHAR(32) NOT NULL,
    group_name VARCHAR(64) NOT NULL,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(server_id, steam_id)
);

-- 封禁记录（用于 HTTP 服务 ban.cfg）
CREATE TABLE IF NOT EXISTS bans (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    steam_id VARCHAR(32) NOT NULL,
    player_name VARCHAR(128) NOT NULL DEFAULT '',
    duration INTEGER NOT NULL DEFAULT 0,
    reason VARCHAR(256) NOT NULL DEFAULT '',
    admin_user VARCHAR(128) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(server_id, steam_id)
);
