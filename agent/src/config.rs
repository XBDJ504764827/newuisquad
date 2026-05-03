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
    pub auto_broadcast_interval_secs: u64,
    pub auto_broadcast_message: String,
    pub welcome_message: String,
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
        let auto_broadcast_interval_secs: u64 = env::var("AUTO_BROADCAST_INTERVAL")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        let auto_broadcast_message = env::var("AUTO_BROADCAST_MESSAGE")
            .unwrap_or_default();
        let welcome_message = env::var("WELCOME_MESSAGE")
            .unwrap_or_default();

        eprintln!("[Config] BACKEND_WS_URL = {}", backend_ws_url);
        eprintln!("[Config] TOKEN          = {}...", &token[..16.min(token.len())]);
        eprintln!("[Config] LOG_FILE_PATH  = {}", log_file_path);
        eprintln!("[Config] RCON           = {}:{}", rcon_host, rcon_port);

        Self { backend_ws_url, token, log_file_path, game_dir, config_dir, rcon_host, rcon_port, rcon_password, auto_broadcast_interval_secs, auto_broadcast_message, welcome_message }
    }
}
