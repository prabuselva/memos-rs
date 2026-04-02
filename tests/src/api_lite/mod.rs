use axum::Router;
use std::sync::Arc;

#[cfg(feature = "lite")]
use crate::AppStateLite as AppState;

#[cfg(all(feature = "embeddings", not(feature = "lite")))]
use crate::AppState;

mod routes;

pub fn create_router() -> Router<Arc<AppState>> {
    routes::create_router()
}
