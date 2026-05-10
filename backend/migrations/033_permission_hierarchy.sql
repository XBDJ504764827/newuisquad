-- 权限组：添加角色继承、模板、管理员标记
ALTER TABLE permission_groups ADD COLUMN IF NOT EXISTS parent_group_id INTEGER REFERENCES permission_groups(id);
ALTER TABLE permission_groups ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE permission_groups ADD COLUMN IF NOT EXISTS is_template BOOLEAN NOT NULL DEFAULT false;

-- 预置权限模板（system 模板，is_template=true 的不会被写入 Admins.cfg）

-- 超级管理员模板
INSERT INTO permission_groups (server_id, group_name, permissions, is_admin, is_template)
SELECT id, '超级管理员模板', 'ui:*,rcon:*', true, true
FROM servers
ON CONFLICT (server_id, group_name) DO NOTHING;

-- 普通管理员模板
INSERT INTO permission_groups (server_id, group_name, permissions, is_admin, is_template)
SELECT id, '管理员模板', 'ui:dashboard:view,ui:players:view,ui:players:warn,ui:players:kick,ui:bans:view,ui:bans:create,ui:console:view,ui:console:execute,ui:audit_logs:view,rcon:chat,rcon:kick,rcon:ban,rcon:manageserver,rcon:cameraman,rcon:forceteamchange,rcon:teamchange,rcon:changemap,rcon:pause,rcon:disbandsquad,rcon:removefromsquad,rcon:canseeadminchat', true, true
FROM servers
ON CONFLICT (server_id, group_name) DO NOTHING;

-- 观察员模板
INSERT INTO permission_groups (server_id, group_name, permissions, is_admin, is_template)
SELECT id, '观察员模板', 'ui:dashboard:view,ui:players:view,ui:bans:view,ui:feeds:view,ui:audit_logs:view,rcon:canseeadminchat', false, true
FROM servers
ON CONFLICT (server_id, group_name) DO NOTHING;
