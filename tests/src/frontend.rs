use axum::{
    body::Body,
    response::Redirect,
    routing::{get, get_service},
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[cfg(not(feature = "lite"))]
use crate::AppState;

#[cfg(feature = "lite")]
use crate::state_lite::AppStateLite as AppStateLite;

#[cfg(feature = "embed-frontend")]
use rust_embed::RustEmbed;

#[cfg(not(feature = "lite"))]
pub fn create_app_router() -> Router<Arc<AppState>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let dist_path = std::path::Path::new(&manifest_dir).join("dist");

    #[cfg(feature = "embed-frontend")]
    if has_embedded_assets() {
        return Router::new()
            .route("/", get(|| async { Redirect::temporary("/app/") }))
            .route("/app", get(|| async { Redirect::temporary("/app/") }))
            .nest_service("/app/", get_service(EmbeddedServeDir::new()))
            .route("/app/login", get(login_fallback))
            .route("/app/register", get(login_fallback))
            .route("/app/forgot-password", get(login_fallback))
            .route("/app/reset-password/:token", get(login_fallback))
            .route("/health", get(health));
    }

    if dist_path.exists() {
        Router::new()
            .route("/", get(|| async { Redirect::temporary("/app/") }))
            .route("/app", get(|| async { Redirect::temporary("/app/") }))
            .nest_service(
                "/app/",
                get_service(ServeDir::new(&dist_path).append_index_html_on_directories(true)),
            )
            .route("/app/login", get(login_fallback_non_embedded))
            .route("/app/register", get(login_fallback_non_embedded))
            .route("/app/forgot-password", get(login_fallback_non_embedded))
            .route(
                "/app/reset-password/:token",
                get(login_fallback_non_embedded),
            )
            .route("/health", get(health))
    } else {
        Router::new()
            .route("/", get(|| async { "Memos RS - Note Taking API" }))
            .route("/health", get(health))
    }
}

#[cfg(feature = "lite")]
pub fn create_app_router() -> Router<Arc<crate::state_lite::AppStateLite>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let dist_path = std::path::Path::new(&manifest_dir).join("dist");

    #[cfg(feature = "embed-frontend")]
    if has_embedded_assets() {
        return Router::new()
            .route("/", get(|| async { Redirect::temporary("/app/") }))
            .route("/app", get(|| async { Redirect::temporary("/app/") }))
            .nest_service("/app/", get_service(EmbeddedServeDir::new()))
            .route("/app/login", get(login_fallback))
            .route("/app/register", get(login_fallback))
            .route("/app/forgot-password", get(login_fallback))
            .route("/app/reset-password/:token", get(login_fallback))
            .route("/health", get(health));
    }

    if dist_path.exists() {
        Router::new()
            .route("/", get(|| async { Redirect::temporary("/app/") }))
            .route("/app", get(|| async { Redirect::temporary("/app/") }))
            .nest_service(
                "/app/",
                get_service(ServeDir::new(&dist_path).append_index_html_on_directories(true)),
            )
            .route("/app/login", get(login_fallback_non_embedded))
            .route("/app/register", get(login_fallback_non_embedded))
            .route("/app/forgot-password", get(login_fallback_non_embedded))
            .route(
                "/app/reset-password/:token",
                get(login_fallback_non_embedded),
            )
            .route("/health", get(health))
    } else {
        Router::new()
            .route("/", get(|| async { "Memos RS Lite - Note Taking API" }))
            .route("/health", get(health))
    }
}

#[cfg(feature = "embed-frontend")]
fn has_embedded_assets() -> bool {
    FrontendAssets::iter().next().is_some()
}

#[cfg(feature = "embed-frontend")]
#[derive(RustEmbed)]
#[folder = "dist/"]
struct FrontendAssets;

async fn health() -> &'static str {
    "OK"
}

async fn login_fallback_non_embedded() -> axum::response::Response {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let dist_path = std::path::Path::new(&manifest_dir).join("dist");

    if let Ok(file) = std::fs::read(dist_path.join("index.html")) {
        let body = Body::from(file);
        axum::response::Response::new(body)
    } else {
        axum::response::Response::new(Body::from("Not found"))
    }
}

#[cfg(feature = "embed-frontend")]
async fn login_fallback() -> axum::response::Response {
    if let Some(asset) = FrontendAssets::get("index.html") {
        let mime_type = get_mime_type("index.html");
        let body = Body::from(asset.data);
        let mut response = axum::response::Response::new(body);
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static(mime_type),
        );
        response
    } else {
        axum::response::Response::new(Body::from("Not found"))
    }
}

#[allow(dead_code)]
fn get_mime_type(path: &str) -> &'static str {
    match path.split('.').next_back() {
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("html") => "text/html",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain",
        Some("xml") => "application/xml",
        _ => "application/octet-stream",
    }
}

#[cfg(feature = "embed-frontend")]
#[derive(Clone)]
pub struct EmbeddedServeDir;

#[cfg(feature = "embed-frontend")]
impl Default for EmbeddedServeDir {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "embed-frontend")]
impl EmbeddedServeDir {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "embed-frontend")]
impl tower::Service<axum::http::Request<Body>> for EmbeddedServeDir {
    type Response = axum::response::Response;
    type Error = std::convert::Infallible;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: axum::http::Request<Body>) -> Self::Future {
        let path = req.uri().path().to_string();

        // Don't handle API routes or health check
        if path.starts_with("/api") || path == "/health" {
            let response = axum::response::Response::new(Body::from("Not found"));
            return Box::pin(std::future::ready(Ok(response)));
        }

        let file_path = if path.is_empty() || path == "/" {
            "index.html"
        } else {
            path.trim_start_matches('/')
        };

        if let Some(asset) = FrontendAssets::get(file_path) {
            let mime_type = get_mime_type(file_path);
            let body = Body::from(asset.data);
            let mut response = axum::response::Response::new(body);
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(mime_type),
            );
            Box::pin(std::future::ready(Ok(response)))
        } else {
            let response = axum::response::Response::new(Body::from("Not found"));
            Box::pin(std::future::ready(Ok(response)))
        }
    }
}
