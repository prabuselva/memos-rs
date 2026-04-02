use axum::Router;
use std::sync::Arc;

pub mod routes;

#[cfg(not(feature = "lite"))]
pub mod chat;

#[cfg(not(feature = "lite"))]
pub mod embeddings;

#[cfg(not(feature = "lite"))]
pub mod llm;

#[cfg(not(feature = "lite"))]
use crate::AppState;

pub fn create_router() -> Router<Arc<AppState>> {
    routes::create_router()
}
