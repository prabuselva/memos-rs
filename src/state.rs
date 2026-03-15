use crate::config::Config;
use crate::db::Database;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Mutex<Database>>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let db = Database::new(&config).await?;
        Ok(Self {
            config: Arc::new(config),
            db: Arc::new(Mutex::new(db)),
        })
    }
}