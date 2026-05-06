-- 添加道歉关键字和TK广播消息字段
ALTER TABLE tk_settings
    ADD COLUMN IF NOT EXISTS apology_keyword VARCHAR(64) NOT NULL DEFAULT 'sry',
    ADD COLUMN IF NOT EXISTS tk_broadcast_message TEXT;
