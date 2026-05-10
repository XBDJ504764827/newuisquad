-- 伤害通知: 添加消息模板和发送模式
ALTER TABLE damage_notify_settings
    ADD COLUMN IF NOT EXISTS message_mode VARCHAR(32) NOT NULL DEFAULT 'warning_related',
    ADD COLUMN IF NOT EXISTS hit_layout TEXT NOT NULL DEFAULT '[命中] {{attacker}} 对 {{victim}} 造成了 {{damage}} 点伤害，使用 {{weapon}}',
    ADD COLUMN IF NOT EXISTS kill_layout TEXT NOT NULL DEFAULT '[击杀] {{attacker}} 击杀了 {{victim}}，造成 {{damage}} 点伤害，使用 {{weapon}}';
