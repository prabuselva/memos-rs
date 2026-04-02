use crate::models::{Note, Notebook, NotebookNode, PasswordRecovery, Session, User};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Row, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::embeddings::{EmbeddingCache, EmbeddingModel};
use crate::vector::{VectorStore, BM25};

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
    pub vector_store: Option<Arc<VectorStore>>,
    pub embedding_model: Option<Arc<dyn EmbeddingModel + Send + Sync>>,
    pub embedding_cache: Option<Arc<crate::embeddings::EmbeddingCache>>,
}

impl Database {
    pub async fn new(config: &crate::config::Config) -> Result<Self> {
        let path = config
            .database
            .path
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from(".memos-rs/data.sqlite"));

        let path_str = path.display().to_string();
        info!("Database path: {}", path_str);

        let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        debug!("Parent directory: {}", parent.display());
        std::fs::create_dir_all(parent).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create database directory '{}': {}",
                parent.display(),
                e
            )
        })?;
        debug!("Directory created successfully");

        // Create empty database file if it doesn't exist
        if !std::path::Path::exists(&path) {
            std::fs::write(&path, "")?;
            debug!("Created database file");
        }

        let pool_url = format!("sqlite://{}", path_str);
        info!("Connecting to database: {}", pool_url);
        let pool = Pool::<Sqlite>::connect(&pool_url).await?;
        info!("Database connection established");

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT,
                updated_at TEXT,
                is_active INTEGER DEFAULT 1,
                metadata TEXT
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                session_token TEXT NOT NULL,
                created_at TEXT,
                expires_at TEXT,
                is_valid INTEGER DEFAULT 1,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS password_recovery (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                token TEXT NOT NULL,
                created_at TEXT,
                expires_at TEXT,
                is_used INTEGER DEFAULT 0,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                content_html TEXT,
                notebook_id TEXT,
                parent_id TEXT,
                created_at TEXT,
                updated_at TEXT,
                is_favorite INTEGER DEFAULT 0,
                is_archived INTEGER DEFAULT 0,
                tags TEXT,
                metadata TEXT,
                user_id TEXT,
                FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE SET NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
            );
            CREATE TABLE IF NOT EXISTS notebooks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                created_at TEXT,
                updated_at TEXT,
                user_id TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
            );
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                created_at TEXT,
                user_id TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
            );
            CREATE TABLE IF NOT EXISTS note_tags (
                note_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (note_id, tag_id),
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&pool)
        .await?;

        // Add metadata column if it doesn't exist (for existing databases)
        sqlx::query(
            r#"
            ALTER TABLE users ADD COLUMN metadata TEXT
            "#,
        )
        .execute(&pool)
        .await
        .ok(); // Ignore error if column already exists

        let vector_store = if config.vector.enabled {
            Some(Arc::new(VectorStore::new(&config.vector.url).await?))
        } else {
            None
        };

        Ok(Self {
            pool,
            vector_store,
            embedding_model: None,
            embedding_cache: None,
        })
    }

    pub async fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub fn with_embedding_model(
        self,
        model: Arc<dyn EmbeddingModel + Send + Sync>,
        cache: Arc<EmbeddingCache>,
    ) -> Self {
        Self {
            embedding_model: Some(model),
            embedding_cache: Some(cache),
            ..self
        }
    }

    pub async fn create(&self, note: Note) -> Result<Note> {
        let id = note.id.clone();
        let title = &note.title;
        let content = &note.content;
        let content_html = &note.content_html;
        let parent_id = &note.parent_id;
        let created_at = note.created_at.to_rfc3339();
        let updated_at = note.updated_at.to_rfc3339();
        let parent_id_str = parent_id.as_deref();
        let is_favorite = note.is_favorite as i32;
        let is_archived = note.is_archived as i32;
        let tags = serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string());
        let metadata = serde_json::to_string(&note.metadata).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            r#"
            INSERT INTO notes (id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(title)
        .bind(content)
        .bind(content_html)
        .bind(parent_id_str)
        .bind(created_at)
        .bind(updated_at)
        .bind(is_favorite)
        .bind(is_archived)
        .bind(tags)
        .bind(metadata)
        .execute(&self.pool)
        .await?;

        #[cfg(feature = "vector-db")]
        {
            if let (Some(vector_store), Some(user_id)) = (&self.vector_store, note.user_id.clone()) {
                if vector_store.get_bm25_state(&user_id).await?.is_none() {
                    if let Ok(bm25) = self.initialize_bm25_from_notes(&user_id).await {
                        debug!("[create::DEBUG] Initialized BM25 with {} notes", bm25);
                    }
                }
            }
        }

        Ok(note)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Note> {
        let row = sqlx::query(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id, notebook_id
            FROM notes WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let created_at_str: String = row.get(5);
        let updated_at_str: String = row.get(6);
        let tags_str: String = row.get(9);
        let metadata_str: String = row.get(10);
        let user_id: Option<String> = row.get(11);
        let notebook_id: Option<String> = row.get(12);

        Ok(Note {
            id: row.get(0),
            title: row.get(1),
            content: row.get(2),
            content_html: row.get(3),
            parent_id: row.get(4),
            notebook_id,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            is_favorite: row.get::<i32, _>(7) == 1,
            is_archived: row.get::<i32, _>(8) == 1,
            tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
            metadata: serde_json::from_str(&metadata_str)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
            user_id,
        })
    }

    pub async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Note>> {
        let limit_val = limit.unwrap_or(100) as i64;
        let offset_val = offset.unwrap_or(0) as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id, notebook_id
            FROM notes WHERE is_archived = 0
            ORDER BY updated_at DESC LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit_val)
        .bind(offset_val)
        .fetch_all(&self.pool)
        .await?;

        let notes = rows
            .into_iter()
            .map(|row| {
                let created_at_str: String = row.get(5);
                let updated_at_str: String = row.get(6);
                let tags_str: String = row.get(9);
                let metadata_str: String = row.get(10);
                let user_id: Option<String> = row.get(11);
                let notebook_id: Option<String> = row.get(12);

                Note {
                    id: row.get(0),
                    title: row.get(1),
                    content: row.get(2),
                    content_html: row.get(3),
                    parent_id: row.get(4),
                    notebook_id,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    is_favorite: row.get::<i32, _>(7) == 1,
                    is_archived: row.get::<i32, _>(8) == 1,
                    tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
                    metadata: serde_json::from_str(&metadata_str)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                    user_id,
                }
            })
            .collect();

        Ok(notes)
    }

    pub async fn update(&self, note: Note) -> Result<Note> {
        let mut note = note;
        note.updated_at = Utc::now();

        let updated_at = note.updated_at.to_rfc3339();
        let parent_id_str = note.parent_id.as_deref();
        let id = note.id.clone();
        let title = note.title.clone();
        let content = note.content.clone();
        let content_html = &note.content_html;
        let is_favorite = note.is_favorite as i32;
        let is_archived = note.is_archived as i32;
        let tags = serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string());
        let metadata = serde_json::to_string(&note.metadata).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            r#"
            UPDATE notes SET title = ?, content = ?, content_html = ?, parent_id = ?, 
            updated_at = ?, is_favorite = ?, is_archived = ?, tags = ?, metadata = ?
            WHERE id = ?
            "#,
        )
        .bind(title)
        .bind(content)
        .bind(content_html)
        .bind(parent_id_str)
        .bind(updated_at)
        .bind(is_favorite)
        .bind(is_archived)
        .bind(tags)
        .bind(metadata)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(note)
    }

     pub async fn delete(&self, id: &str) -> Result<()> {
        let note = self.get_by_id(id).await?;

        sqlx::query(
            r#"
            UPDATE notes SET is_archived = 1, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if let (Some(vector_store), Some(user_id)) = (&self.vector_store, note.user_id) {
            vector_store.delete_note(&user_id, id).await.ok();
            
            #[cfg(feature = "vector-db")]
            {
                let _ = self.delete_from_bm25(&user_id, id).await;
            }
        }

        Ok(())
    }

    pub async fn upsert_note_to_vector(&self, note: &Note) -> Result<()> {
        if let (Some(vector_store), Some(model), Some(cache)) = (
            &self.vector_store,
            &self.embedding_model,
            &self.embedding_cache,
        ) {
            let text_for_embedding = format!("{} {}", note.title, note.content);

            let embedding = if let Some(cached) = cache.get(&text_for_embedding) {
                cached.clone()
            } else {
                let start_time = std::time::Instant::now();
                let mut embedding = model.embed(&text_for_embedding)?;
                let elapsed = start_time.elapsed();
                debug!(
                    "[EMBED] Generated embedding for {} chars in {:?}, dimension={}",
                    text_for_embedding.len(),
                    elapsed,
                    embedding.len()
                );

                let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                embedding = embedding.iter().map(|x| x / norm).collect::<Vec<_>>();

                cache.set(&text_for_embedding, &embedding)?;
                embedding
            };

            let payload = serde_json::json!({
                "id": note.id,
                "title": note.title,
                "content": note.content,
                "content_html": note.content_html,
                "user_id": note.user_id,
                "notebook_id": note.notebook_id,
                "parent_id": note.parent_id,
                "tags": note.tags,
                "metadata": note.metadata,
                "created_at": note.created_at.to_rfc3339(),
                "updated_at": note.updated_at.to_rfc3339(),
                "is_favorite": note.is_favorite,
                "is_archived": note.is_archived
            });

            let user_id = note.user_id.as_ref().ok_or_else(|| anyhow::anyhow!("Note must have user_id"))?;
            vector_store
                .upsert_note(user_id, &note.id, embedding, payload)
                .await?;

            let bm25_text = format!("{} {}", note.title, note.content);
            match vector_store.get_bm25_state(user_id).await {
                Ok(Some(mut bm25)) => {
                    bm25.add_document(&bm25_text, note.id.clone());
                    if let Err(e) = vector_store.save_bm25_state(user_id, &bm25).await {
                        debug!("[upsert_note_to_vector::DEBUG] Failed to save BM25 state: {}", e);
                    }
                }
                Ok(None) => {
                    let mut bm25 = crate::vector::BM25::new(1.5, 0.75);
                    bm25.add_document(&bm25_text, note.id.clone());
                    if let Err(e) = vector_store.save_bm25_state(user_id, &bm25).await {
                        debug!("[upsert_note_to_vector::DEBUG] Failed to save BM25 state: {}", e);
                    }
                }
                Err(e) => {
                    debug!("[upsert_note_to_vector::DEBUG] Failed to get BM25 state: {}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn delete_notes_by_ids(&self, ids: &[String]) -> Result<usize> {
        let mut deleted_count = 0;

        for id in ids {
            if self.delete(id).await.is_ok() {
                deleted_count += 1;
            }
        }

      Ok(deleted_count)
    }

    #[cfg(feature = "vector-db")]
    pub async fn initialize_bm25_from_notes(&self, user_id: &str) -> Result<usize> {
        let notes = self.get_notes_by_user(user_id, None, None).await?;

        if notes.is_empty() {
            return Ok(0);
        }

        if let Some(vector_store) = &self.vector_store {
            let mut bm25 = crate::vector::BM25::new(1.5, 0.75);
            let mut processed = 0;

            for note in &notes {
                let text = format!("{} {}", note.title, note.content);
                bm25.add_document(&text, note.id.clone());
                processed += 1;
            }

            if let Err(e) = vector_store.save_bm25_state(user_id, &bm25).await {
                error!("[initialize_bm25_from_notes::ERROR] Failed to save BM25 state: {}", e);
                return Err(anyhow::anyhow!("Failed to initialize BM25 state"));
            }

            Ok(processed)
        } else {
            Ok(0)
        }
    }

    #[cfg(feature = "vector-db")]
    pub async fn delete_from_bm25(&self, user_id: &str, note_id: &str) -> Result<()> {
        if let Some(vector_store) = &self.vector_store {
            if let Some(mut bm25) = vector_store.get_bm25_state(user_id).await? {
                if bm25.delete_document(note_id) {
                    if let Err(e) = vector_store.save_bm25_state(user_id, &bm25).await {
                        error!("[delete_from_bm25::ERROR] Failed to save BM25 state: {}", e);
                        return Err(anyhow::anyhow!("Failed to delete from BM25"));
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, is_active
            FROM users WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;

        let created_at_str: String = row.get(4);
        let updated_at_str: String = row.get(5);

        Ok(User {
            id: row.get(0),
            username: row.get(1),
            email: row.get(2),
            password_hash: row.get(3),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            is_active: row.get::<i32, _>(6) == 1,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        })
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, is_active
            FROM users WHERE email = ?
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        let created_at_str: String = row.get(4);
        let updated_at_str: String = row.get(5);

        Ok(User {
            id: row.get(0),
            username: row.get(1),
            email: row.get(2),
            password_hash: row.get(3),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            is_active: row.get::<i32, _>(6) == 1,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        })
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<User> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, is_active, metadata
            FROM users WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        let created_at_str: String = row.get(4);
        let updated_at_str: String = row.get(5);
        let metadata: Option<String> = row.get(7);

        Ok(User {
            id: row.get(0),
            username: row.get(1),
            email: row.get(2),
            password_hash: row.get(3),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            is_active: row.get::<i32, _>(6) == 1,
            metadata: metadata
                .and_then(|m| serde_json::from_str(&m).ok())
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        })
    }

    pub async fn create_user(&self, user: User) -> Result<User> {
        let id = user.id.clone();
        let username = user.username.clone();
        let email = user.email.clone();
        let password_hash = user.password_hash.clone();
        let created_at = user.created_at.to_rfc3339();
        let updated_at = user.updated_at.to_rfc3339();
        let is_active = user.is_active as i32;
        let metadata = serde_json::to_string(&user.metadata).unwrap_or_else(|_| "{}".to_string());

        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(created_at)
        .bind(updated_at)
        .bind(is_active)
        .bind(metadata)
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create_session(&self, session: Session) -> Result<Session> {
        let id = session.id.clone();
        let user_id = session.user_id.clone();
        let session_token = session.session_token.clone();
        let created_at = session.created_at.to_rfc3339();
        let expires_at = session.expires_at.to_rfc3339();
        let is_valid = session.is_valid as i32;

        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, session_token, created_at, expires_at, is_valid)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(session_token)
        .bind(created_at)
        .bind(expires_at)
        .bind(is_valid)
        .execute(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get_session(&self, session_token: &str) -> Result<Session> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, session_token, created_at, expires_at, is_valid
            FROM sessions WHERE session_token = ? AND is_valid = 1
            "#,
        )
        .bind(session_token)
        .fetch_one(&self.pool)
        .await?;

        let created_at_str: String = row.get(3);
        let expires_at_str: String = row.get(4);

        Ok(Session {
            id: row.get(0),
            user_id: row.get(1),
            session_token: row.get(2),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            expires_at: DateTime::parse_from_rfc3339(&expires_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            is_valid: row.get::<i32, _>(5) == 1,
        })
    }

    pub async fn invalidate_session(&self, session_token: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE sessions SET is_valid = 0 WHERE session_token = ?
            "#,
        )
        .bind(session_token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_password_recovery(
        &self,
        recovery: PasswordRecovery,
    ) -> Result<PasswordRecovery> {
        let id = recovery.id.clone();
        let user_id = recovery.user_id.clone();
        let token = recovery.token.clone();
        let created_at = recovery.created_at.to_rfc3339();
        let expires_at = recovery.expires_at.to_rfc3339();
        let is_used = recovery.is_used as i32;

        sqlx::query(
            r#"
            INSERT INTO password_recovery (id, user_id, token, created_at, expires_at, is_used)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(token)
        .bind(created_at)
        .bind(expires_at)
        .bind(is_used)
        .execute(&self.pool)
        .await?;

        Ok(recovery)
    }

    pub async fn get_password_recovery(&self, token: &str) -> Result<PasswordRecovery> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, token, created_at, expires_at, is_used
            FROM password_recovery WHERE token = ? AND is_used = 0
            "#,
        )
        .bind(token)
        .fetch_one(&self.pool)
        .await?;

        let created_at_str: String = row.get(3);
        let expires_at_str: String = row.get(4);

        Ok(PasswordRecovery {
            id: row.get(0),
            user_id: row.get(1),
            token: row.get(2),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            expires_at: DateTime::parse_from_rfc3339(&expires_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            is_used: row.get::<i32, _>(5) == 1,
        })
    }

    pub async fn mark_password_recovery_as_used(&self, token: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE password_recovery SET is_used = 1 WHERE token = ?
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_notes_by_ids(&self, note_ids: &[String]) -> Result<Vec<Note>> {
        if note_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<&str> = note_ids.iter().map(|_| "?").collect();
        let placeholder_str = placeholders.join(", ");

        let query_str = format!(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id, notebook_id
            FROM notes WHERE is_archived = 0 AND id IN ({})
            "#,
            placeholder_str
        );

        let mut query = sqlx::query(&query_str);

        for note_id in note_ids {
            query = query.bind(note_id);
        }

        let rows = query.fetch_all(&self.pool).await?;

        let notes = rows
            .into_iter()
            .map(|row| {
                let created_at_str: String = row.get(5);
                let updated_at_str: String = row.get(6);
                let tags_str: String = row.get(9);
                let metadata_str: String = row.get(10);
                let user_id_opt: Option<String> = row.get(11);
                let notebook_id_opt: Option<String> = row.get(12);

                Note {
                    id: row.get(0),
                    title: row.get(1),
                    content: row.get(2),
                    content_html: row.get(3),
                    parent_id: row.get(4),
                    notebook_id: notebook_id_opt,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    is_favorite: row.get::<i32, _>(8) == 1,
                    is_archived: row.get::<i32, _>(8) == 1,
                    tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
                    metadata: serde_json::from_str(&metadata_str)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                    user_id: user_id_opt,
                }
            })
            .collect();

        Ok(notes)
    }

    pub async fn search_notes_by_vector(
        &self,
        user_id: &str,
        embedding: &[f32],
        limit: u32,
    ) -> Result<Vec<Note>, anyhow::Error> {
        if let Some(vector_store) = &self.vector_store {
            let results = vector_store
                .search_notes(embedding.to_vec(), user_id, limit)
                .await?;

            let note_ids: Vec<String> = results
                .iter()
                .filter_map(|r| {
                    r.get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string())
                })
                .collect();

            if !note_ids.is_empty() {
                return self.get_notes_by_ids(&note_ids).await;
            }
        }

        Ok(Vec::new())
    }

    pub async fn search_notes_by_vector_with_scores(
        &self,
        user_id: &str,
        embedding: &[f32],
        limit: u32,
    ) -> Result<Vec<(Note, f32)>, anyhow::Error> {
        if let Some(vector_store) = &self.vector_store {
            let results = vector_store
                .search_notes_with_scores(embedding.to_vec(), user_id, limit)
                .await?;

            let mut notes_with_scores = Vec::new();
            for point in results {
                if let Some(payload) = point.payload {
                    if let Some(id_str) = payload.get("id").and_then(|v| v.as_str()) {
                        if let Ok(note) = self.get_by_id(id_str).await {
                            notes_with_scores.push((note, point.score));
                        }
                    }
                }
            }

            return Ok(notes_with_scores);
        }

        Ok(Vec::new())
    }

    pub async fn get_notes_by_user(
        &self,
        user_id: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Note>> {
        let limit_val = limit.unwrap_or(100) as i64;
        let offset_val = offset.unwrap_or(0) as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id, notebook_id
            FROM notes WHERE is_archived = 0 AND user_id = ?
            ORDER BY updated_at DESC LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(limit_val)
        .bind(offset_val)
        .fetch_all(&self.pool)
        .await?;

        let notes = rows
            .into_iter()
            .map(|row| {
                let created_at_str: String = row.get(5);
                let updated_at_str: String = row.get(6);
                let tags_str: String = row.get(9);
                let metadata_str: String = row.get(10);
                let user_id_opt: Option<String> = row.get(11);
                let notebook_id_opt: Option<String> = row.get(12);

                Note {
                    id: row.get(0),
                    title: row.get(1),
                    content: row.get(2),
                    content_html: row.get(3),
                    parent_id: row.get(4),
                    notebook_id: notebook_id_opt,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    is_favorite: row.get::<i32, _>(8) == 1,
                    is_archived: row.get::<i32, _>(8) == 1,
                    tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
                    metadata: serde_json::from_str(&metadata_str)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                    user_id: user_id_opt,
                }
            })
            .collect();

        Ok(notes)
    }

    pub async fn search_notes_by_user(
        &self,
        user_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<Note>> {
        let search_pattern = format!("%{}%", query);
        let limit_val = limit as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id, notebook_id
            FROM notes 
            WHERE is_archived = 0 AND user_id = ? 
            AND (title LIKE ? OR content LIKE ?)
            ORDER BY 
                CASE 
                    WHEN title LIKE ? THEN 1 
                    ELSE 2 
                END,
                updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(user_id)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(limit_val)
        .fetch_all(&self.pool)
        .await?;

        let notes = rows
            .into_iter()
            .map(|row| {
                let created_at_str: String = row.get(5);
                let updated_at_str: String = row.get(6);
                let tags_str: String = row.get(9);
                let metadata_str: String = row.get(10);
                let user_id_opt: Option<String> = row.get(11);
                let notebook_id_opt: Option<String> = row.get(12);

                Note {
                    id: row.get(0),
                    title: row.get(1),
                    content: row.get(2),
                    content_html: row.get(3),
                    parent_id: row.get(4),
                    notebook_id: notebook_id_opt,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    is_favorite: row.get::<i32, _>(8) == 1,
                    is_archived: row.get::<i32, _>(8) == 1,
                    tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
                    metadata: serde_json::from_str(&metadata_str)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                    user_id: user_id_opt,
                }
            })
            .collect();

        Ok(notes)
    }

    pub async fn create_note_with_user(&self, note: Note) -> Result<Note> {
        let id = note.id.clone();
        let title = &note.title;
        let content = &note.content;
        let content_html = &note.content_html;
        let notebook_id = &note.notebook_id;
        let parent_id = &note.parent_id;
        let created_at = note.created_at.to_rfc3339();
        let updated_at = note.updated_at.to_rfc3339();
        let notebook_id_str = notebook_id.as_deref();
        let parent_id_str = parent_id.as_deref();
        let is_favorite = note.is_favorite as i32;
        let is_archived = note.is_archived as i32;
        let tags = serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string());
        let metadata = serde_json::to_string(&note.metadata).unwrap_or_else(|_| "{}".to_string());
        let user_id = note.user_id.as_deref();

        sqlx::query(
            r#"
            INSERT INTO notes (id, title, content, content_html, notebook_id, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(title)
        .bind(content)
        .bind(content_html)
        .bind(notebook_id_str)
        .bind(parent_id_str)
        .bind(created_at)
        .bind(updated_at)
        .bind(is_favorite)
        .bind(is_archived)
        .bind(tags)
        .bind(metadata)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if let Err(e) = self.upsert_note_to_vector(&note).await {
            error!("Warning: Failed to upsert note to vector: {}", e);
        }

        Ok(note)
    }

    pub async fn update_note_with_user(&self, note: Note) -> Result<Note> {
        let mut note = note;
        note.updated_at = Utc::now();

        let updated_at = note.updated_at.to_rfc3339();
        let notebook_id_str = note.notebook_id.as_deref();
        let parent_id_str = note.parent_id.as_deref();
        let id = note.id.clone();
        let title = note.title.clone();
        let content = note.content.clone();
        let content_html = &note.content_html;
        let is_favorite = note.is_favorite as i32;
        let is_archived = note.is_archived as i32;
        let tags = serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string());
        let metadata = serde_json::to_string(&note.metadata).unwrap_or_else(|_| "{}".to_string());
        let user_id = note.user_id.as_deref();

        sqlx::query(
            r#"
            UPDATE notes SET title = ?, content = ?, content_html = ?, notebook_id = ?, parent_id = ?, 
            updated_at = ?, is_favorite = ?, is_archived = ?, tags = ?, metadata = ?, user_id = ?
            WHERE id = ?
            "#,
        )
        .bind(title)
        .bind(content)
        .bind(content_html)
        .bind(notebook_id_str)
        .bind(parent_id_str)
        .bind(updated_at)
        .bind(is_favorite)
        .bind(is_archived)
        .bind(tags)
        .bind(metadata)
        .bind(user_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if let Err(e) = self.upsert_note_to_vector(&note).await {
            error!("Warning: Failed to upsert note to vector: {}", e);
        }

        Ok(note)
    }

    pub async fn delete_user_notes(&self, user_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM notes WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if let Some(vector_store) = &self.vector_store {
            if let Err(e) = vector_store.delete_user_data(user_id).await {
                error!("Warning: Failed to delete user data from vector store: {}", e);
            }
        }

        Ok(())
    }

    pub async fn update_user(&self, user: User) -> Result<User> {
        let updated_at = user.updated_at.to_rfc3339();
        let id = user.id.clone();
        let username = user.username.clone();
        let email = user.email.clone();
        let password_hash = user.password_hash.clone();
        let is_active = user.is_active as i32;

        sqlx::query(
            r#"
            UPDATE users SET username = ?, email = ?, password_hash = ?, updated_at = ?, is_active = ?
            WHERE id = ?
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(updated_at)
        .bind(is_active)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_user_search_mode(
        &self,
        user_id: &str,
        search_mode: &str,
    ) -> Result<(), anyhow::Error> {
        let metadata = serde_json::json!({
            "search_mode": search_mode
        });

        sqlx::query(
            r#"
            UPDATE users SET metadata = ? WHERE id = ?
            "#,
        )
        .bind(Some(&serde_json::to_string(&metadata)?))
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_metadata(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let row = sqlx::query(
            r#"
            SELECT metadata FROM users WHERE id = ?
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let metadata_str: Option<String> = row.get(0);
        let metadata: serde_json::Value = if let Some(metadata_str) = metadata_str {
            serde_json::from_str(&metadata_str)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()))
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        Ok(metadata)
    }

    pub async fn update_user_llm_settings(
        &self,
        user_id: &str,
        llm_settings: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        let metadata = serde_json::json!({
            "llm_settings": llm_settings
        });

        sqlx::query(
            r#"
            UPDATE users SET metadata = ? WHERE id = ?
            "#,
        )
        .bind(Some(&serde_json::to_string(&metadata)?))
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_llm_settings(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let row = sqlx::query(
            r#"
            SELECT metadata FROM users WHERE id = ?
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let metadata_str: Option<String> = row.get(0);
        let metadata: serde_json::Value = if let Some(metadata_str) = metadata_str {
            serde_json::from_str(&metadata_str)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()))
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        let llm_settings = metadata.get("llm_settings").cloned().unwrap_or_else(|| {
            serde_json::json!({
                "provider": "ollama",
                "url": "http://localhost:11434",
                "api_key": null,
                "model": "llama2",
                "temperature": 0.7,
                "max_tokens": 2048
            })
        });

        Ok(llm_settings)
    }

    pub async fn list_notebooks_by_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<Notebook>, anyhow::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, parent_id, created_at, updated_at, user_id
            FROM notebooks WHERE user_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let notebooks = rows
            .into_iter()
            .map(|row| {
                let created_at_str: String = row.get(3);
                let updated_at_str: String = row.get(4);
                let user_id_opt: Option<String> = row.get(5);

                Notebook {
                    id: row.get(0),
                    name: row.get(1),
                    parent_id: row.get(2),
                    user_id: user_id_opt,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                }
            })
            .collect();

        Ok(notebooks)
    }

    pub async fn get_notebook_by_id(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<Notebook, anyhow::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, name, parent_id, created_at, updated_at, user_id
            FROM notebooks WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let created_at_str: String = row.get(3);
        let updated_at_str: String = row.get(4);
        let user_id_opt: Option<String> = row.get(5);

        Ok(Notebook {
            id: row.get(0),
            name: row.get(1),
            parent_id: row.get(2),
            user_id: user_id_opt,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
        })
    }

    pub async fn get_notebook_by_name(
        &self,
        name: &str,
        user_id: &str,
    ) -> Result<Notebook, anyhow::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, name, parent_id, created_at, updated_at, user_id
            FROM notebooks WHERE name = ? AND user_id = ?
            "#,
        )
        .bind(name)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let created_at_str: String = row.get(3);
        let updated_at_str: String = row.get(4);
        let user_id_opt: Option<String> = row.get(5);

        Ok(Notebook {
            id: row.get(0),
            name: row.get(1),
            parent_id: row.get(2),
            user_id: user_id_opt,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
        })
    }

    pub async fn create_notebook(&self, notebook: Notebook) -> Result<Notebook, anyhow::Error> {
        let id = notebook.id.clone();
        let name = notebook.name.clone();
        let parent_id_str = notebook.parent_id.as_deref();
        let created_at = notebook.created_at.to_rfc3339();
        let updated_at = notebook.updated_at.to_rfc3339();
        let user_id = notebook.user_id.as_deref();

        sqlx::query(
            r#"
            INSERT INTO notebooks (id, name, parent_id, created_at, updated_at, user_id)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(parent_id_str)
        .bind(created_at)
        .bind(updated_at)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(notebook)
    }

    pub async fn update_notebook(&self, notebook: Notebook) -> Result<Notebook, anyhow::Error> {
        let mut notebook = notebook;
        notebook.updated_at = Utc::now();

        let updated_at = notebook.updated_at.to_rfc3339();
        let parent_id_str = notebook.parent_id.as_deref();
        let id = notebook.id.clone();
        let name = notebook.name.clone();

        sqlx::query(
            r#"
            UPDATE notebooks SET name = ?, parent_id = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(parent_id_str)
        .bind(updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(notebook)
    }

    pub async fn delete_notebook(&self, id: &str) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            DELETE FROM notebooks WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_notes_by_notebook_id(
        &self,
        notebook_id: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Note>, anyhow::Error> {
        let limit_val = limit.unwrap_or(100) as i64;
        let offset_val = offset.unwrap_or(0) as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id, notebook_id
            FROM notes WHERE is_archived = 0 AND notebook_id = ?
            ORDER BY updated_at DESC LIMIT ? OFFSET ?
            "#,
        )
        .bind(notebook_id)
        .bind(limit_val)
        .bind(offset_val)
        .fetch_all(&self.pool)
        .await?;

        let notes = rows
            .into_iter()
            .map(|row| {
                let created_at_str: String = row.get(5);
                let updated_at_str: String = row.get(6);
                let tags_str: String = row.get(9);
                let metadata_str: String = row.get(10);
                let user_id_opt: Option<String> = row.get(11);
                let notebook_id_opt: Option<String> = row.get(12);

                Note {
                    id: row.get(0),
                    title: row.get(1),
                    content: row.get(2),
                    content_html: row.get(3),
                    parent_id: row.get(4),
                    notebook_id: notebook_id_opt,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    is_favorite: row.get::<i32, _>(7) == 1,
                    is_archived: row.get::<i32, _>(8) == 1,
                    tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
                    metadata: serde_json::from_str(&metadata_str)
                        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                    user_id: user_id_opt,
                }
            })
            .collect();

        Ok(notes)
    }

    pub async fn get_notebooks_tree(
        &self,
        user_id: &str,
    ) -> Result<Vec<NotebookNode>, anyhow::Error> {
        let notebooks = self.list_notebooks_by_user(user_id).await?;
        let notes = self.get_notes_by_user(user_id, None, None).await?;

        let mut notebook_map: HashMap<String, NotebookNode> = HashMap::new();

        for notebook in &notebooks {
            notebook_map.insert(notebook.id.clone(), NotebookNode::new(notebook.clone()));
        }

        for note in &notes {
            if let Some(notebook_id) = &note.notebook_id {
                if let Some(node) = notebook_map.get_mut(notebook_id) {
                    node.notes.push(note.clone());
                }
            }
        }

        let mut root_nodes: Vec<NotebookNode> = Vec::new();

        for (_, node) in &notebook_map {
            if node.notebook.parent_id.is_some() {
                continue;
            }
            root_nodes.push(node.clone());
        }

        for (_, node) in &notebook_map {
            if let Some(ref parent_id) = node.notebook.parent_id {
                if let Some(parent_node) =
                    root_nodes.iter_mut().find(|n| n.notebook.id == *parent_id)
                {
                    parent_node.children.push(node.clone());
                }
            }
        }

        Ok(root_nodes)
    }

    #[async_recursion::async_recursion]
    pub async fn get_folder_contents(
        &self,
        notebook_id: Option<&str>,
        user_id: &str,
    ) -> Result<NotebookNode, anyhow::Error> {
        let all_notebooks = self.list_notebooks_by_user(user_id).await?;
        let all_notes = self.get_notes_by_user(user_id, None, None).await?;

        let root_notebooks: Vec<Notebook> = if let Some(id) = notebook_id {
            all_notebooks
                .iter()
                .filter(|n| n.parent_id.as_deref() == Some(id))
                .cloned()
                .collect()
        } else {
            all_notebooks
                .iter()
                .filter(|n| n.parent_id.is_none())
                .cloned()
                .collect()
        };

        let root_notes: Vec<Note> = if let Some(id) = notebook_id {
            all_notes
                .iter()
                .filter(|n| n.notebook_id.as_deref() == Some(id))
                .cloned()
                .collect()
        } else {
            all_notes
                .iter()
                .filter(|n| n.notebook_id.is_none())
                .cloned()
                .collect()
        };

        let root_name = notebook_id
            .and_then(|id| {
                all_notebooks
                    .iter()
                    .find(|n| n.id == id)
                    .map(|n| n.name.clone())
            })
            .unwrap_or_else(|| "Unsorted".to_string());

        let mut node = NotebookNode::new(Notebook::new(root_name));
        node.notes = root_notes;

        for subfolder in &root_notebooks {
            let child = self
                .get_folder_contents(Some(&subfolder.id), user_id)
                .await?;
            node.children.push(child);
        }

        Ok(node)
    }
}
