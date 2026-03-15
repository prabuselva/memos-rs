use sqlx::{Pool, Sqlite, Row};
use anyhow::Result;
use crate::models::{Note, User, Session, PasswordRecovery};
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(config: &crate::config::Config) -> Result<Self> {
        let path = config.database.path.clone()
            .unwrap_or_else(|| std::path::PathBuf::from(".memos-rs/data.sqlite"));
        
        let path_str = path.display().to_string();
        eprintln!("Database path: {}", path_str);
        
        let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        eprintln!("Parent directory: {}", parent.display());
        std::fs::create_dir_all(parent).map_err(|e| anyhow::anyhow!("Failed to create database directory '{}': {}", parent.display(), e))?;
        eprintln!("Directory created successfully");
        
        // Create empty database file if it doesn't exist
        if !std::path::Path::exists(&path) {
            std::fs::write(&path, "")?;
            eprintln!("Created database file");
        }
        
        let pool = Pool::<Sqlite>::connect(&format!("sqlite://{}", path_str)).await?;
        
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
                is_active INTEGER DEFAULT 1
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
                parent_id TEXT,
                created_at TEXT,
                updated_at TEXT,
                is_favorite INTEGER DEFAULT 0,
                is_archived INTEGER DEFAULT 0,
                tags TEXT,
                metadata TEXT,
                user_id TEXT,
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
        
        Ok(Self { pool })
    }

    pub async fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub async fn create(&self, note: Note) -> Result<Note> {
        let id = note.id.clone();
        let title = &note.title;
        let content = &note.content;
        let content_html = &note.content_html;
        let parent_id = &note.parent_id;
        let created_at = note.created_at.to_rfc3339();
        let updated_at = note.updated_at.to_rfc3339();
        let parent_id_str = parent_id.as_ref().map(|s| s.as_str());
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

        Ok(note)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Note> {
        let row = sqlx::query(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id
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

        Ok(Note {
            id: row.get(0),
            title: row.get(1),
            content: row.get(2),
            content_html: row.get(3),
            parent_id: row.get(4),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow::anyhow!("Date parsing error: {}", e))?,
            is_favorite: row.get::<i32, _>(7) == 1,
            is_archived: row.get::<i32, _>(8) == 1,
            tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
            metadata: serde_json::from_str(&metadata_str).unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
            user_id,
        })
    }

    pub async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Note>> {
        let limit_val = limit.unwrap_or(100) as i64;
        let offset_val = offset.unwrap_or(0) as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id
            FROM notes WHERE is_archived = 0
            ORDER BY updated_at DESC LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit_val)
        .bind(offset_val)
        .fetch_all(&self.pool)
        .await?;

        let notes = rows.into_iter().map(|row| {
            let created_at_str: String = row.get(5);
            let updated_at_str: String = row.get(6);
            let tags_str: String = row.get(9);
            let metadata_str: String = row.get(10);
            let user_id: Option<String> = row.get(11);

            Note {
                id: row.get(0),
                title: row.get(1),
                content: row.get(2),
                content_html: row.get(3),
                parent_id: row.get(4),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                is_favorite: row.get::<i32, _>(7) == 1,
                is_archived: row.get::<i32, _>(8) == 1,
                tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
                metadata: serde_json::from_str(&metadata_str).unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                user_id,
            }
        }).collect();

        Ok(notes)
    }

    pub async fn update(&self, note: Note) -> Result<Note> {
        let mut note = note;
        note.updated_at = Utc::now();

        let updated_at = note.updated_at.to_rfc3339();
        let parent_id_str = note.parent_id.as_ref().map(|s| s.as_str());
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
        sqlx::query(
            r#"
            UPDATE notes SET is_archived = 1, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

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

    pub async fn create_user(&self, user: User) -> Result<User> {
        let id = user.id.clone();
        let username = &user.username;
        let email = &user.email;
        let password_hash = &user.password_hash;
        let created_at = user.created_at.to_rfc3339();
        let updated_at = user.updated_at.to_rfc3339();
        let is_active = user.is_active as i32;

        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(created_at)
        .bind(updated_at)
        .bind(is_active)
        .execute(&self.pool)
        .await?;

        Ok(user)
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
        })
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<User> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, is_active
            FROM users WHERE id = ?
            "#,
        )
        .bind(id)
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
        })
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

    pub async fn create_password_recovery(&self, recovery: PasswordRecovery) -> Result<PasswordRecovery> {
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

    pub async fn get_notes_by_user(&self, user_id: &str, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Note>> {
        let limit_val = limit.unwrap_or(100) as i64;
        let offset_val = offset.unwrap_or(0) as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id
            FROM notes WHERE is_archived = 0 AND user_id = ?
            ORDER BY updated_at DESC LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(limit_val)
        .bind(offset_val)
        .fetch_all(&self.pool)
        .await?;

        let notes = rows.into_iter().map(|row| {
            let created_at_str: String = row.get(5);
            let updated_at_str: String = row.get(6);
            let tags_str: String = row.get(9);
            let metadata_str: String = row.get(10);
            let user_id_opt: Option<String> = row.get(11);

            Note {
                id: row.get(0),
                title: row.get(1),
                content: row.get(2),
                content_html: row.get(3),
                parent_id: row.get(4),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                is_favorite: row.get::<i32, _>(7) == 1,
                is_archived: row.get::<i32, _>(8) == 1,
                tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
                metadata: serde_json::from_str(&metadata_str).unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                user_id: user_id_opt,
            }
        }).collect();

        Ok(notes)
    }

    pub async fn search_notes_by_user(&self, user_id: &str, query: &str, limit: u32) -> Result<Vec<Note>> {
        let search_pattern = format!("%{}%", query);
        let limit_val = limit as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id
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

        let notes = rows.into_iter().map(|row| {
            let created_at_str: String = row.get(5);
            let updated_at_str: String = row.get(6);
            let tags_str: String = row.get(9);
            let metadata_str: String = row.get(10);
            let user_id_opt: Option<String> = row.get(11);

            Note {
                id: row.get(0),
                title: row.get(1),
                content: row.get(2),
                content_html: row.get(3),
                parent_id: row.get(4),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                is_favorite: row.get::<i32, _>(7) == 1,
                is_archived: row.get::<i32, _>(8) == 1,
                tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
                metadata: serde_json::from_str(&metadata_str).unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                user_id: user_id_opt,
            }
        }).collect();

        Ok(notes)
    }

    pub async fn create_note_with_user(&self, note: Note) -> Result<Note> {
        let id = note.id.clone();
        let title = &note.title;
        let content = &note.content;
        let content_html = &note.content_html;
        let parent_id = &note.parent_id;
        let created_at = note.created_at.to_rfc3339();
        let updated_at = note.updated_at.to_rfc3339();
        let parent_id_str = parent_id.as_ref().map(|s| s.as_str());
        let is_favorite = note.is_favorite as i32;
        let is_archived = note.is_archived as i32;
        let tags = serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string());
        let metadata = serde_json::to_string(&note.metadata).unwrap_or_else(|_| "{}".to_string());
        let user_id = note.user_id.as_deref();

        sqlx::query(
            r#"
            INSERT INTO notes (id, title, content, content_html, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(note)
    }

    pub async fn update_note_with_user(&self, note: Note) -> Result<Note> {
        let mut note = note;
        note.updated_at = Utc::now();

        let updated_at = note.updated_at.to_rfc3339();
        let parent_id_str = note.parent_id.as_ref().map(|s| s.as_str());
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
            UPDATE notes SET title = ?, content = ?, content_html = ?, parent_id = ?, 
            updated_at = ?, is_favorite = ?, is_archived = ?, tags = ?, metadata = ?, user_id = ?
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
        .bind(user_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

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
}