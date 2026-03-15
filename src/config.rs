use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontendConfig {
    #[serde(default)]
    pub embedded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    pub frontend: FrontendConfig,
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
pub struct StorageConfig {
    pub attachments_dir: String,
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
        }
    }
}
