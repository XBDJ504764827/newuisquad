-- 伤害通知: 添加击倒通知开关
ALTER TABLE damage_notify_settings
    ADD COLUMN IF NOT EXISTS notify_kill BOOLEAN NOT NULL DEFAULT true;
