use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub backend_ws_url: String,
    pub token: String,
    pub log_file_path: String,
    pub game_dir: String,
    pub config_dir: String,
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

        eprintln!("[Config] BACKEND_WS_URL = {}", backend_ws_url);
        eprintln!("[Config] TOKEN          = {}...", &token[..16.min(token.len())]);
        eprintln!("[Config] LOG_FILE_PATH  = {}", log_file_path);
        eprintln!("[Config] GAME_DIR       = {}", game_dir);
        eprintln!("[Config] CONFIG_DIR     = {}", config_dir);

        Self { backend_ws_url, token, log_file_path, game_dir, config_dir }
    }
}
