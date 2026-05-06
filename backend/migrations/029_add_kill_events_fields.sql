-- 击倒记录功能完善：新增 event_type 区分伤害/击倒/阵亡，补充受害者 ID 字段
ALTER TABLE kill_events ADD COLUMN IF NOT EXISTS event_type VARCHAR(16) NOT NULL DEFAULT 'damage';
ALTER TABLE kill_events ADD COLUMN IF NOT EXISTS victim_eos VARCHAR(64) NOT NULL DEFAULT '';
ALTER TABLE kill_events ADD COLUMN IF NOT EXISTS victim_steam64 VARCHAR(32) NOT NULL DEFAULT '';

-- 回填已有数据
UPDATE kill_events SET event_type = 'wound' WHERE is_kill = true AND attacker_name != '';
UPDATE kill_events SET event_type = 'death' WHERE is_kill = true AND attacker_name = '';

CREATE INDEX IF NOT EXISTS idx_kill_events_type ON kill_events(event_type);
CREATE INDEX IF NOT EXISTS idx_kill_events_victim ON kill_events(victim_steam64);
