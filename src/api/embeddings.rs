use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::embeddings::EmbeddingModel;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct EmbedResponse {
    pub embedding: Vec<f32>,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct EmbedRequest {
    pub text: String,
}

pub fn router<M: EmbeddingModel + Clone + 'static>(model: Arc<M>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/embed", post(embed_text))
        .route("/embed-batch", post(embed_batch))
        .with_state(model)
}

async fn embed_text(
    State(model): State<Arc<dyn EmbeddingModel>>,
    Json(req): Json<EmbedRequest>,
) -> Result<Json<EmbedResponse>, (StatusCode, String)> {
    info!("[embed_text] Requested for text: {}", req.text);
    if req.text.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Text cannot be empty".to_string()));
    }

    match model.embed(&req.text) {
        Ok(embedding) => {
            debug!(
                "[embed_text] Got embedding with {} elements",
                embedding.len()
            );
            debug!(
                "[embed_text] First 10 elements: {:?}",
                &embedding[0..10.min(embedding.len())]
            );
            debug!(
                "[embed_text] Last 10 elements: {:?}",
                &embedding[(embedding.len() - 10).max(0)..]
            );
            let response = Json(EmbedResponse {
                embedding,
                model: model.name().to_string(),
            });
            debug!("[embed_text] Response created");
            Ok(response)
        }
        Err(e) => {
            error!("[embed_text] Error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

async fn embed_batch(
    State(model): State<Arc<dyn EmbeddingModel>>,
    Json(req): Json<BatchEmbedRequest>,
) -> Result<Json<BatchEmbedResponse>, (StatusCode, String)> {
    if req.texts.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "At least one text required".to_string(),
        ));
    }

    let texts: Vec<&str> = req.texts.iter().map(|s| s.as_str()).collect();

    match model.embed_batch(&texts) {
        Ok(embeddings) => Ok(Json(BatchEmbedResponse {
            embeddings,
            model: model.name().to_string(),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Debug, Deserialize)]
pub struct BatchEmbedRequest {
    pub texts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchEmbedResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub model: String,
}
