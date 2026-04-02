use crate::AppState;
use axum::Router;
use std::sync::Arc;

use crate::embeddings::EmbeddingModel;

pub fn create_app_router() -> Router<Arc<AppState>> {
    crate::frontend::create_app_router()
}

pub fn create_app_router_with_model<M: EmbeddingModel + Clone + 'static>(
    model: Arc<M>,
) -> Router<Arc<AppState>> {
    let base_router = create_app_router();
    let api_router = Router::new()
        .nest("/embeddings", crate::api::embeddings::router(model.clone()))
        .nest("/chat", crate::api::chat::router(model.clone()))
        .nest("/llm", crate::api::llm::router(model))
        .merge(crate::api::create_router());
    base_router.nest("/api/v1", api_router)
}

fn llm_router_with_model<M: EmbeddingModel + Clone + 'static>(
    model: Arc<M>,
) -> Router<Arc<AppState>> {
    Router::new()
        .nest("/embeddings", crate::api::embeddings::router(model.clone()))
        .nest("/chat", crate::api::chat::router(model.clone()))
        .nest("/llm", crate::api::llm::router(model))
}
