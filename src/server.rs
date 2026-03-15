use crate::AppState;
use axum::Router;
use std::sync::Arc;

pub fn create_app_router() -> Router<Arc<AppState>> {
    crate::frontend::create_app_router()
}

async fn root() -> &'static str {
    "Memos RS - Note Taking API"
}

async fn health() -> &'static str {
    "OK"
}