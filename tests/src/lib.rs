#![recursion_limit = "512"]

pub mod api_lite;
#[cfg(feature = "embeddings")]
pub mod api;
pub mod config;
#[cfg(feature = "embeddings")]
pub mod embeddings;
pub mod db_lite;

#[cfg(feature = "embeddings")]
pub mod db;
#[cfg(feature = "lite")]
pub use db_lite as db;

pub mod frontend;
pub mod import_export;
#[cfg(feature = "llm")]
pub mod llm;
pub mod markdown;
pub mod models;
pub mod repositories;
#[cfg(feature = "embeddings")]
pub mod server;
pub mod server_lite;
pub mod services;
#[cfg(feature = "embeddings")]
pub mod state;
pub mod state_lite;
#[cfg(feature = "embeddings")]
pub mod test_data;
pub mod utils;
#[cfg(any(feature = "vector-db", feature = "embeddings"))]
pub mod vector;
pub mod version;

pub use models::note::{Reference, SearchMetadata};

pub use config::Config;

pub use frontend::create_app_router;

pub use models::note::{Note, NoteReference};

pub use models::{PasswordRecovery, Session, User};

pub use services::auth_service;

pub use utils::auth_utils;

pub use crate::version::{VERSION, VERSION_SHORT};

#[cfg(feature = "embeddings")]
pub use api::chat::call_llm_api;

#[cfg(all(feature = "embeddings", feature = "lite"))]
compile_error!("lite and embeddings/vector-db/llm features are mutually exclusive");

#[cfg(all(feature = "llm", not(feature = "embeddings")))]
compile_error!("llm feature requires embeddings feature");

#[cfg(feature = "lite")]
pub use db_lite::Database as Database;

#[cfg(all(feature = "embeddings", not(feature = "lite")))]
pub use db::Database as Database;

#[cfg(feature = "embeddings")]
pub use embeddings::{BERTModel, EmbeddingCache, EmbeddingModel, ModelDownloader};

#[cfg(feature = "embeddings")]
pub use server::create_app_router_with_model;

#[cfg(feature = "lite")]
pub use state_lite::AppStateLite as AppStateLite;

#[cfg(feature = "lite")]
pub use state_lite::AppStateLite as AppState;

#[cfg(all(feature = "embeddings", not(feature = "lite")))]
pub use state::AppState as AppState;

#[cfg(feature = "embeddings")]
pub use test_data::{
    generate_random_notes, get_wikipedia_page, import_wikipedia_notes,
    initialize_vector_store_with_notes, search_wikipedia, seed_test_data,
};

#[cfg(feature = "lite")]
pub use api_lite::create_router as create_router_lite;

#[cfg(feature = "lite")]
pub use server_lite::create_app_router_lite;
