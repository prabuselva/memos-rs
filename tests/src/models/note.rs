use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub content_html: Option<String>,
    pub notebook_id: Option<String>,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub user_id: Option<String>,
}

impl Note {
    pub fn new(title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            content,
            content_html: None,
            notebook_id: None,
            parent_id: None,
            created_at: now,
            updated_at: now,
            is_favorite: false,
            is_archived: false,
            tags: Vec::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            user_id: None,
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_notebook(mut self, notebook_id: String) -> Self {
        self.notebook_id = Some(notebook_id);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_content_html(mut self, content_html: String) -> Self {
        self.content_html = Some(content_html);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Notebook {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            parent_id: None,
            user_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            color: None,
        }
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteReference {
    pub id: u64,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub score: f32,
    pub distance: f32,
    pub user_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub id: u64,
    pub note_id: String,
    pub title: String,
    pub content_snippet: String,
    pub score: f32,
    pub distance: f32,
    pub used_in_response: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub query: String,
    pub vector_search_time_ms: u64,
    pub llm_generation_time_ms: u64,
    pub total_tokens: usize,
    pub retrieved_count: usize,
    pub filtered_count: usize,
    pub hybrid_search: bool,
    pub model: String,
}

pub type NoteWithScore = (Note, f32);

#[derive(Debug, Error)]
pub enum NoteError {
    #[error("Note not found: {0}")]
    NotFound(String),
    #[error("Notebook not found: {0}")]
    NotebookNotFound(String),
    #[error("Tag not found: {0}")]
    TagNotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

pub type NoteResult<T> = Result<T, NoteError>;
