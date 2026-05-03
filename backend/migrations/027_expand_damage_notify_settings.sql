-- 扩展 damage_notify_settings 表，添加伤害阈值和通知类型配置
ALTER TABLE damage_notify_settings
    ADD COLUMN IF NOT EXISTS min_damage DOUBLE PRECISION NOT NULL DEFAULT 20.0,
    ADD COLUMN IF NOT EXISTS notify_tk BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS notify_damage BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS notify_high_damage BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS high_damage_threshold DOUBLE PRECISION NOT NULL DEFAULT 80.0;
