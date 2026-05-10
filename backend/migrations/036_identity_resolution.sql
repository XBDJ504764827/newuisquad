-- 玩家身份识别表（Union-Find 并查集 → Steam/EOS/名称关联）
CREATE TABLE IF NOT EXISTS player_identities (
    canonical_id VARCHAR(64) PRIMARY KEY,
    primary_steam_id VARCHAR(32) NOT NULL DEFAULT '',
    primary_eos_id VARCHAR(64) NOT NULL DEFAULT '',
    primary_name VARCHAR(128) NOT NULL DEFAULT '',
    all_steam_ids TEXT[] NOT NULL DEFAULT '{}',
    all_eos_ids TEXT[] NOT NULL DEFAULT '{}',
    all_names TEXT[] NOT NULL DEFAULT '{}',
    total_sessions INTEGER NOT NULL DEFAULT 0,
    first_seen TIMESTAMPTZ,
    last_seen TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS player_identity_lookup (
    identifier_type VARCHAR(16) NOT NULL, -- 'steam', 'eos', 'name'
    identifier_value VARCHAR(128) NOT NULL,
    canonical_id VARCHAR(64) NOT NULL REFERENCES player_identities(canonical_id) ON DELETE CASCADE,
    PRIMARY KEY (identifier_type, identifier_value)
);

CREATE INDEX IF NOT EXISTS idx_identity_lookup_canonical ON player_identity_lookup(canonical_id);
