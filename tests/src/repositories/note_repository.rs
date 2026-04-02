use crate::db::Database;
use crate::models::Note;
use async_trait::async_trait;

#[async_trait]
pub trait NoteRepository {
    async fn create(&self, note: Note) -> Result<Note, String>;
    async fn get_by_id(&self, id: &str) -> Result<Note, String>;
    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Note>, String>;
    async fn update(&self, note: Note) -> Result<Note, String>;
    async fn delete(&self, id: &str) -> Result<(), String>;
}

#[async_trait]
impl NoteRepository for Database {
    async fn create(&self, note: Note) -> Result<Note, String> {
        Database::create(self, note)
            .await
            .map_err(|e| e.to_string())
    }

    async fn get_by_id(&self, id: &str) -> Result<Note, String> {
        Database::get_by_id(self, id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Note>, String> {
        Database::list(self, limit, offset)
            .await
            .map_err(|e| e.to_string())
    }

    async fn update(&self, note: Note) -> Result<Note, String> {
        Database::update(self, note)
            .await
            .map_err(|e| e.to_string())
    }

    async fn delete(&self, id: &str) -> Result<(), String> {
        Database::delete(self, id).await.map_err(|e| e.to_string())
    }
}
