use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub backend_ws_url: String,
    pub token: String,
    pub log_file_path: String,
    pub game_dir: String,
    pub config_dir: String,
    pub rcon_host: String,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub rcon_poll_interval_secs: u64,
    pub auto_broadcast_interval_secs: u64,
    pub auto_broadcast_message: String,
    pub welcome_message: String,
    pub admins_cfg_path: String,
    // 批量上传配置
    pub batch_max_size: usize,
    pub batch_max_delay_ms: u64,
    pub batch_flush_interval_ms: u64,
    // 压缩配置
    pub compression_enabled: bool,
    pub compression_level: i32,
    pub compression_min_bytes: usize,
    // Offset 持久化
    pub offset_file_path: Option<String>,
    pub offset_flush_interval_secs: u64,
    // 配置下发
    pub cfg_report_enabled: bool,
    pub cfg_report_interval_secs: u64,
    pub cfg_pull_enabled: bool,
    pub cfg_pull_interval_ms: u64,
    pub cfg_pull_limit: usize,
    // Agent 标识
    pub agent_id: String,
}

impl Config {
    pub fn from_env() -> Self {
        let backend_ws_url = env::var("BACKEND_WS_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:8000/agent/connect".into());
        let token = env::var("TOKEN")
            .unwrap_or_else(|_| "no-token".into());
        let log_file_path = env::var("LOG_FILE_PATH")
            .unwrap_or_else(|_| r"C:\game\server.log".into());
        let game_dir = env::var("GAME_DIR")
            .unwrap_or_else(|_| r"C:\game\".into());
        let config_dir = env::var("CONFIG_DIR")
            .unwrap_or_else(|_| game_dir.clone());
        let rcon_host = env::var("RCON_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let rcon_port: u16 = env::var("RCON_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(28016);
        let rcon_password = env::var("RCON_PASSWORD").unwrap_or_default();
        let rcon_poll_interval_secs: u64 = env::var("RCON_POLL_INTERVAL")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(3);
        let auto_broadcast_interval_secs: u64 = env::var("AUTO_BROADCAST_INTERVAL")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        let auto_broadcast_message = env::var("AUTO_BROADCAST_MESSAGE")
            .unwrap_or_default();
        let welcome_message = env::var("WELCOME_MESSAGE")
            .unwrap_or_default();
        let admins_cfg_path = env::var("ADMINS_CFG_PATH")
            .unwrap_or_else(|_| format!("{}/Admins.cfg", config_dir));

        eprintln!("[Config] BACKEND_WS_URL = {}", backend_ws_url);
        eprintln!("[Config] TOKEN          = {}...", &token[..16.min(token.len())]);
        eprintln!("[Config] LOG_FILE_PATH  = {}", log_file_path);
        eprintln!("[Config] RCON           = {}:{}", rcon_host, rcon_port);
        eprintln!("[Config] ADMINS_CFG     = {}", admins_cfg_path);

        // 批量上传配置
        let batch_max_size: usize = env::var("BATCH_MAX_SIZE")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(200);
        let batch_max_delay_ms: u64 = env::var("BATCH_MAX_DELAY_MS")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(20);
        let batch_flush_interval_ms: u64 = env::var("BATCH_FLUSH_INTERVAL_MS")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(1000);

        // 压缩配置
        let compression_enabled: bool = env::var("COMPRESSION_ENABLED")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(true);
        let compression_level: i32 = env::var("COMPRESSION_LEVEL")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(1);
        let compression_min_bytes: usize = env::var("COMPRESSION_MIN_BYTES")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(4096);

        // Offset 持久化
        let offset_file_path: Option<String> = env::var("OFFSET_FILE_PATH")
            .ok().filter(|s| !s.is_empty());
        let offset_flush_interval_secs: u64 = env::var("OFFSET_FLUSH_INTERVAL_SECS")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(1);

        // 配置下发
        let cfg_report_enabled: bool = env::var("CFG_REPORT_ENABLED")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(true);
        let cfg_report_interval_secs: u64 = env::var("CFG_REPORT_INTERVAL_SECS")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(600);
        let cfg_pull_enabled: bool = env::var("CFG_PULL_ENABLED")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(true);
        let cfg_pull_interval_ms: u64 = env::var("CFG_PULL_INTERVAL_MS")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(500);
        let cfg_pull_limit: usize = env::var("CFG_PULL_LIMIT")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(4);

        // Agent ID
        let agent_id = env::var("AGENT_ID")
            .unwrap_or_else(|_| {
                // 默认使用主机名
                env::var("COMPUTERNAME")
                    .or_else(|_| env::var("HOSTNAME"))
                    .unwrap_or_else(|_| "edge-agent".into())
            });

        eprintln!("[Config] BATCH_MAX_SIZE = {}", batch_max_size);
        eprintln!("[Config] COMPRESSION     = {} (level: {}, min: {})", compression_enabled, compression_level, compression_min_bytes);
        eprintln!("[Config] AGENT_ID       = {}", agent_id);

        Self {
            backend_ws_url,
            token,
            log_file_path,
            game_dir,
            config_dir,
            rcon_host,
            rcon_port,
            rcon_password,
            rcon_poll_interval_secs,
            auto_broadcast_interval_secs,
            auto_broadcast_message,
            welcome_message,
            admins_cfg_path,
            batch_max_size,
            batch_max_delay_ms,
            batch_flush_interval_ms,
            compression_enabled,
            compression_level,
            compression_min_bytes,
            offset_file_path,
            offset_flush_interval_secs,
            cfg_report_enabled,
            cfg_report_interval_secs,
            cfg_pull_enabled,
            cfg_pull_interval_ms,
            cfg_pull_limit,
            agent_id,
        }
    }
}
