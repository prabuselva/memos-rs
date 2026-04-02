use axum::{extract::State, response::Json, routing::get, Router};
use serde::Serialize;
use std::sync::Arc;

use crate::embeddings::model::EmbeddingModel;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ModelStatus {
    pub name: String,
    pub dimension: usize,
    pub loaded: bool,
    pub ready: bool,
}

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub model_name: String,
    pub embedding_dim: usize,
    pub max_sequence_length: usize,
    pub cache_enabled: bool,
}

pub fn router<M: EmbeddingModel + Clone + 'static>(model: Arc<M>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/status", get(get_status))
        .route("/config", get(get_config))
        .with_state(model)
}

async fn get_status(State(model): State<Arc<dyn EmbeddingModel>>) -> Json<ModelStatus> {
    Json(ModelStatus {
        name: model.name().to_string(),
        dimension: model.dimension(),
        loaded: true,
        ready: true,
    })
}

async fn get_config(State(model): State<Arc<dyn EmbeddingModel>>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        model_name: model.name().to_string(),
        embedding_dim: model.dimension(),
        max_sequence_length: 512,
        cache_enabled: true,
    })
}
