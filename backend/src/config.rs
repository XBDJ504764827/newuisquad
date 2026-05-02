use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub log_file_path: String,
    pub steam_api_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://newsquad:newsquad@192.168.0.62:5432/newsquad".into()),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8000),
            log_file_path: env::var("LOG_FILE_PATH")
                .unwrap_or_else(|_| "/var/log/game/server.log".into()),
            steam_api_key: env::var("STEAM_API_KEY")
                .unwrap_or_default(),
        }
    }
}
