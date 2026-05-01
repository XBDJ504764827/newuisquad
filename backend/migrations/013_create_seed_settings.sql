CREATE TABLE IF NOT EXISTS seed_settings (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL UNIQUE REFERENCES servers(id),
    enabled BOOLEAN NOT NULL DEFAULT false,
    player_threshold INTEGER NOT NULL DEFAULT 20,
    vehicle_claim BOOLEAN NOT NULL DEFAULT true,
    vehicle_fill BOOLEAN NOT NULL DEFAULT true,
    deploy_restrict BOOLEAN NOT NULL DEFAULT false,
    kit_restrict BOOLEAN NOT NULL DEFAULT false,
    heavy_vehicle_require BOOLEAN NOT NULL DEFAULT false,
    respawn_timer BOOLEAN NOT NULL DEFAULT true,
    use_enemy_vehicle BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
