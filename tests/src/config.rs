use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontendConfig {
    #[serde(default)]
    pub embedded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    pub enabled: bool,
    pub url: String,
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
    #[serde(default = "default_embedding_dim")]
    pub embedding_dim: usize,
    #[serde(default = "default_model_cache_dir")]
    pub model_cache_dir: String,
    #[serde(default = "default_enable_cache")]
    pub enable_cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "compact".to_string()
}

impl Config {
    pub fn get_tokenizer_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.vector.model_cache_dir)
            .join("all-MiniLM-L6-v2")
            .join("tokenizer.json")
    }

    pub fn get_model_dir(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.vector.model_cache_dir).join("all-MiniLM-L6-v2")
    }
}

fn default_embedding_model() -> String {
    "all-MiniLM-L6-v2".to_string()
}

fn default_embedding_dim() -> usize {
    384
}

fn default_model_cache_dir() -> String {
    ".memos-rs/models".to_string()
}

fn default_enable_cache() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    #[serde(default = "default_llm_provider")]
    pub provider: String,
    #[serde(default = "default_llm_url")]
    pub url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_llm_model")]
    pub model: String,
    #[serde(default = "default_llm_temperature")]
    pub temperature: f64,
    #[serde(default = "default_llm_max_tokens")]
    pub max_tokens: i32,
}

fn default_llm_provider() -> String {
    "openai".to_string()
}

fn default_llm_url() -> String {
    "http://localhost:11434/v1".to_string()
}

fn default_llm_model() -> String {
    "llama3".to_string()
}

fn default_llm_temperature() -> f64 {
    0.7
}

fn default_llm_max_tokens() -> i32 {
    2048
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub attachments_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    pub frontend: FrontendConfig,
    pub vector: VectorConfig,
    pub search: SearchConfig,
    pub rag: RagConfig,
    pub llm: LLMConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub kind: DatabaseKind,
    pub path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseKind {
    SQLite,
    PostgreSQL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_search_mode")]
    pub default_mode: String,
}

fn default_search_mode() -> String {
    "sql".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    #[serde(default = "default_rag_top_k")]
    pub top_k: usize,
    #[serde(default = "default_rag_relevance_threshold")]
    pub relevance_threshold: f32,
    #[serde(default = "default_rag_max_context_tokens")]
    pub max_context_tokens: usize,
    #[serde(default = "default_rag_enable_citations")]
    pub enable_citations: bool,
    #[serde(default = "default_rag_hybrid_search")]
    pub hybrid_search: bool,
    #[serde(default = "default_rag_max_notes")]
    pub max_notes: usize,
    #[serde(default = "default_rag_hybrid_weight")]
    pub hybrid_weight: f32,
    #[serde(default = "default_rag_filter_fields")]
    pub filter_fields: Vec<String>,
}

fn default_rag_top_k() -> usize {
    5
}

fn default_rag_relevance_threshold() -> f32 {
    0.0
}

/// Qdrant vector search threshold documentation:
///
/// The `relevance_threshold` is used to filter search results from Qdrant.
/// Qdrant returns cosine similarity scores between 0.0 and 1.0 where:
/// - 1.0 = perfect match (identical vectors)
/// - 0.0 = no similarity (orthogonal vectors)
///
/// The threshold filters results to only include notes with similarity >= threshold.
///
/// Common threshold values:
/// - 0.0: Return all results (no filtering)
/// - 0.5: Moderate relevance (reasonable balance)
/// - 0.7: High relevance (strict filtering)
/// - 0.9: Very high relevance (very strict filtering)
///
/// Distance conversion: Qdrant returns similarity, but we convert to distance as (1.0 - similarity)
/// where distance ranges from 0.0 (identical) to 1.0 (completely different).
///
/// ## Hybrid Search
///
/// Hybrid search combines vector similarity with keyword-based filters:
/// - **Vector search**: Uses cosine similarity to find semantically similar notes
/// - **Filters**: Filters by payload fields (e.g., user_id, tags, created_at)
/// - **Hybrid weight**: Weight for combining vector scores (0.0 to 1.0)
///   - 0.0 = keyword only, 1.0 = vector only, 0.5 = equal weight
///
/// Configure via `rag.hybrid_search` (boolean) and `rag.filter_fields` (array of field names).

fn default_rag_max_context_tokens() -> usize {
    4000
}

fn default_rag_enable_citations() -> bool {
    true
}

fn default_rag_hybrid_search() -> bool {
    false
}

fn default_rag_max_notes() -> usize {
    10
}

fn default_rag_hybrid_weight() -> f32 {
    0.5
}

fn default_rag_filter_fields() -> Vec<String> {
    vec!["user_id".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub session_duration_days: i64,
    pub password_reset_duration_hours: i64,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub bcrypt_cost: u32,
}

impl Default for Config {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                kind: DatabaseKind::SQLite,
                path: Some(current_dir.join(".memos-rs/data.sqlite")),
            },
            storage: StorageConfig {
                attachments_dir: current_dir
                    .join(".memos-rs/attachments")
                    .to_string_lossy()
                    .to_string(),
            },
            auth: AuthConfig {
                session_duration_days: 7,
                password_reset_duration_hours: 1,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
                bcrypt_cost: 12,
            },
            frontend: FrontendConfig::default(),
            vector: VectorConfig {
                enabled: true,
                url: "http://localhost:6333".to_string(),
                embedding_model: default_embedding_model(),
                embedding_dim: default_embedding_dim(),
                model_cache_dir: default_model_cache_dir(),
                enable_cache: default_enable_cache(),
            },
            search: SearchConfig {
                default_mode: default_search_mode(),
            },
            rag: RagConfig {
                top_k: default_rag_top_k(),
                relevance_threshold: default_rag_relevance_threshold(),
                max_context_tokens: default_rag_max_context_tokens(),
                enable_citations: default_rag_enable_citations(),
                hybrid_search: default_rag_hybrid_search(),
                max_notes: default_rag_max_notes(),
                hybrid_weight: default_rag_hybrid_weight(),
                filter_fields: default_rag_filter_fields(),
            },
            llm: LLMConfig {
                provider: default_llm_provider(),
                url: default_llm_url(),
                api_key: String::new(),
                model: default_llm_model(),
                temperature: default_llm_temperature(),
                max_tokens: default_llm_max_tokens(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                format: default_log_format(),
            },
        }
    }
}
