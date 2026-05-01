use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub backend_ws_url: String,
    pub token: String,
    pub log_file_path: String,
    pub game_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            backend_ws_url: env::var("BACKEND_WS_URL")
                .unwrap_or_else(|_| "ws://127.0.0.1:8000/agent/connect".into()),
            token: env::var("TOKEN")
                .unwrap_or_else(|_| "no-token".into()),
            log_file_path: env::var("LOG_FILE_PATH")
                .unwrap_or_else(|_| r"C:\game\server.log".into()),
            game_dir: env::var("GAME_DIR")
                .unwrap_or_else(|_| r"C:\game\".into()),
        }
    }
}
