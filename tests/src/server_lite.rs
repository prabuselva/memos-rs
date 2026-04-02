use axum::Router;
use std::sync::Arc;

#[cfg(not(feature = "lite"))]
use crate::AppState;

#[cfg(feature = "lite")]
use crate::AppStateLite;

#[cfg(not(feature = "lite"))]
pub fn create_app_router() -> Router<Arc<AppState>> {
    crate::frontend::create_app_router()
}

#[cfg(feature = "lite")]
pub fn create_app_router() -> Router<Arc<AppStateLite>> {
    crate::frontend::create_app_router()
}

#[cfg(feature = "lite")]
pub fn create_app_router_lite() -> Router<Arc<AppStateLite>> {
    let base_router = create_app_router();
    let api_router = Router::new().merge(crate::api_lite::create_router());
    base_router.nest("/api/v1", api_router)
}
