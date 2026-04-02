use crate::config::Config;
use crate::db_lite::Database;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppStateLite {
    pub config: Arc<Config>,
    pub db: Arc<Mutex<Database>>,
}

impl AppStateLite {
    pub async fn new(config: Config) -> Result<Self> {
        let db = Database::new(&config).await?;
        let db = Arc::new(Mutex::new(db));

        Ok(Self {
            config: Arc::new(config),
            db,
        })
    }
}
