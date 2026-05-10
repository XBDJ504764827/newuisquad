-- TK 设置: 添加道歉预窗口和消息模板
ALTER TABLE tk_settings
    ADD COLUMN IF NOT EXISTS apology_pre_window_secs INTEGER NOT NULL DEFAULT 20,
    ADD COLUMN IF NOT EXISTS tk_attacker_msg TEXT NOT NULL DEFAULT '你对队友 {{victim}} 造成了友伤，请在 {{seconds}} 秒内输入 {{keyword}} 道歉',
    ADD COLUMN IF NOT EXISTS tk_victim_msg TEXT NOT NULL DEFAULT '你被队友 {{attacker}} 误伤了',
    ADD COLUMN IF NOT EXISTS tk_broadcast_msg TEXT NOT NULL DEFAULT '[TK] {{attacker}} 误伤了 {{victim}}，请输入 {{keyword}} 道歉';
