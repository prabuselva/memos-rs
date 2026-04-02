use crate::models::{Notebook, Note, NotebookNode};
use crate::db::Database;
use async_trait::async_trait;

#[async_trait]
pub trait NotebookRepository {
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Notebook>, String>;
    async fn get_by_id(&self, id: &str, user_id: &str) -> Result<Notebook, String>;
    async fn create(&self, notebook: Notebook) -> Result<Notebook, String>;
    async fn update(&self, notebook: Notebook) -> Result<Notebook, String>;
    async fn delete(&self, id: &str) -> Result<(), String>;
    async fn get_note_count(&self, notebook_id: &str) -> Result<i64, String>;
    async fn get_notes_by_notebook(&self, notebook_id: &str, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Note>, String>;
    async fn get_notebooks_tree(&self, user_id: &str) -> Result<Vec<NotebookNode>, String>;
}

#[async_trait]
impl NotebookRepository for Database {
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Notebook>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, parent_id, created_at, updated_at, user_id
            FROM notebooks WHERE user_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let notebooks = rows.into_iter().map(|row| {
            let created_at_str: String = row.get(3);
            let updated_at_str: String = row.get(4);
            let user_id_opt: Option<String> = row.get(5);

            Notebook {
                id: row.get(0),
                name: row.get(1),
                parent_id: row.get(2),
                user_id: user_id_opt,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            }
        }).collect();

        Ok(notebooks)
    }

    async fn get_by_id(&self, id: &str, user_id: &str) -> Result<Notebook, String> {
        let row = sqlx::query(
            r#"
            SELECT id, name, parent_id, created_at, updated_at, user_id
            FROM notebooks WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let created_at_str: String = row.get(3);
        let updated_at_str: String = row.get(4);
        let user_id_opt: Option<String> = row.get(5);

        Ok(Notebook {
            id: row.get(0),
            name: row.get(1),
            parent_id: row.get(2),
            user_id: user_id_opt,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| e.to_string())?,
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| e.to_string())?,
        })
    }

    async fn create(&self, notebook: Notebook) -> Result<Notebook, String> {
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
        .await
        .map_err(|e| e.to_string())?;

        Ok(notebook)
    }

    async fn update(&self, notebook: Notebook) -> Result<Notebook, String> {
        let mut notebook = notebook;
        notebook.updated_at = chrono::Utc::now();

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
        .await
        .map_err(|e| e.to_string())?;

        Ok(notebook)
    }

    async fn delete(&self, id: &str) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM notebooks WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn get_note_count(&self, notebook_id: &str) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM notes WHERE notebook_id = ?
            "#,
        )
        .bind(notebook_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(count)
    }

    async fn get_notes_by_notebook(&self, notebook_id: &str, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Note>, String> {
        let limit_val = limit.unwrap_or(100) as i64;
        let offset_val = offset.unwrap_or(0) as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, title, content, content_html, notebook_id, parent_id, created_at, updated_at, is_favorite, is_archived, tags, metadata, user_id
            FROM notes WHERE is_archived = 0 AND notebook_id = ?
            ORDER BY updated_at DESC LIMIT ? OFFSET ?
            "#,
        )
        .bind(notebook_id)
        .bind(limit_val)
        .bind(offset_val)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let notes = rows.into_iter().map(|row| {
            let created_at_str: String = row.get(6);
            let updated_at_str: String = row.get(7);
            let tags_str: String = row.get(10);
            let metadata_str: String = row.get(11);
            let user_id_opt: Option<String> = row.get(12);
            let notebook_id_opt: Option<String> = row.get(4);

            Note {
                id: row.get(0),
                title: row.get(1),
                content: row.get(2),
                content_html: row.get(3),
                notebook_id: notebook_id_opt,
                parent_id: row.get(5),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                is_favorite: row.get::<i32, _>(8) == 1,
                is_archived: row.get::<i32, _>(9) == 1,
                tags: serde_json::from_str(&tags_str).unwrap_or_else(|_| Vec::new()),
                metadata: serde_json::from_str(&metadata_str).unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                user_id: user_id_opt,
            }
        }).collect();

        Ok(notes)
    }

    async fn get_notebooks_tree(&self, user_id: &str) -> Result<Vec<NotebookNode>, String> {
        let notebooks = self.list_by_user(user_id).await?;
        let notes = self.get_notes_by_notebook(user_id, None, None).await.map_err(|e| e.to_string())?;

        let mut notebook_map: std::collections::HashMap<String, NotebookNode> = std::collections::HashMap::new();
        
        for notebook in &notebooks {
            notebook_map.insert(
                notebook.id.clone(),
                NotebookNode::new(notebook.clone()),
            );
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
            if node.parent_id.is_some() {
                continue;
            }
            root_nodes.push(node.clone());
        }
        
        for (_, node) in &notebook_map {
            if let Some(ref parent_id) = node.parent_id {
                if let Some(parent_node) = root_nodes.iter_mut().find(|n| n.id == *parent_id) {
                    parent_node.children.push(node.clone());
                }
            }
        }

        Ok(root_nodes)
    }
}
