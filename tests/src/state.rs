use crate::config::{Config, RagConfig};
use crate::db::Database;

#[cfg(feature = "embeddings")]
use crate::embeddings::EmbeddingCache;

#[cfg(feature = "embeddings")]
use crate::embeddings::EmbeddingModel;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Mutex<Database>>,

    #[cfg(feature = "embeddings")]
    pub embedding_model: Arc<dyn EmbeddingModel + Send + Sync>,

    #[cfg(feature = "embeddings")]
    pub embedding_cache: Arc<EmbeddingCache>,

    pub rag_config: Arc<RagConfig>,
}

impl AppState {
    #[cfg(feature = "embeddings")]
    pub async fn new(config: Config, embedding_model: Arc<dyn EmbeddingModel + Send + Sync>) -> Result<Self> {
        let cache = EmbeddingCache::new(&config.vector.model_cache_dir);
        let db = Database::new(&config).await?;
        let cache = Arc::new(cache);
        let db = db.with_embedding_model(embedding_model.clone(), cache.clone());
        let rag_config = config.rag.clone();

        Ok(Self {
            config: Arc::new(config),
            db: Arc::new(Mutex::new(db)),
            embedding_model,
            embedding_cache: cache,
            rag_config: Arc::new(rag_config),
        })
    }

    #[cfg(not(feature = "embeddings"))]
    pub async fn new(config: Config) -> Result<Self> {
        let db = Database::new(&config).await?;
        let rag_config = config.rag.clone();

        Ok(Self {
            config: Arc::new(config),
            db: Arc::new(Mutex::new(db)),
            rag_config: Arc::new(rag_config),
        })
    }
}
