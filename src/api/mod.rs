use axum::Router;
use std::sync::Arc;
use crate::AppState;

mod routes;

pub fn create_router() -> Router<Arc<AppState>> {
    routes::create_router()
}