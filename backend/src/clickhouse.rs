use std::sync::Arc;
use clickhouse::Client;

pub mod schema;

#[derive(Clone)]
pub struct ClickHousePool {
    client: Arc<Client>,
    pub database: String,
}

impl ClickHousePool {
    pub fn new(url: &str, database: &str, user: Option<&str>, password: Option<&str>) -> Self {
        let mut client = Client::default()
            .with_url(url)
            .with_database(database);

        if let Some(user) = user {
            client = client.with_user(user);
        }
        if let Some(password) = password {
            client = client.with_password(password);
        }

        Self {
            client: Arc::new(client),
            database: database.to_string(),
        }
    }

    pub fn client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }

    pub async fn health_check(&self) -> bool {
        self.client
            .query("SELECT 1")
            .fetch_one::<u8>()
            .await
            .is_ok()
    }
}
