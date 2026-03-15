use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

impl User {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn deactivate(mut self) -> Self {
        self.is_active = false;
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub session_token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_valid: bool,
}

impl Session {
    pub fn new(user_id: String, session_token: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            session_token,
            created_at: Utc::now(),
            expires_at,
            is_valid: true,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn invalidate(mut self) -> Self {
        self.is_valid = false;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordRecovery {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
}

impl PasswordRecovery {
    pub fn new(user_id: String, token: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            token,
            created_at: Utc::now(),
            expires_at,
            is_used: false,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn mark_as_used(mut self) -> Self {
        self.is_used = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token invalid")]
    TokenInvalid,
    #[error("Password mismatch")]
    PasswordMismatch,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Username already exists")]
    UsernameAlreadyExists,
    #[error("Email already exists")]
    EmailAlreadyExists,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Anyhow error: {0}")]
    Any(#[from] anyhow::Error),
}

impl Serialize for AuthError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

pub type AuthResult<T> = Result<T, AuthError>;
