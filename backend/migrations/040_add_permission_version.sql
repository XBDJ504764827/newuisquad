-- 为 admin_users 添加权限版本号字段
-- 每次修改权限时自增，JWT 中携带版本号用于实时失效检测
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS permission_version INT NOT NULL DEFAULT 1;
