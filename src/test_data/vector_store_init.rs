use crate::embeddings::EmbeddingModel;
use crate::models::Note;
use crate::vector::store::VectorStore;
use tracing::info;

pub async fn initialize_vector_store_with_notes(
    vector_store: &VectorStore,
    model: &dyn EmbeddingModel,
    notes: &[Note],
) -> Result<usize, anyhow::Error> {
    let mut processed = 0;

    for note in notes {
        let text_for_embedding = format!("{} {}", note.title, note.content);
        let embedding = model.embed(&text_for_embedding)?;

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
        processed += 1;
    }

    Ok(processed)
}

pub async fn seed_test_data(
    vector_store: &VectorStore,
    model: &dyn EmbeddingModel,
    notes: &[Note],
) -> Result<(), anyhow::Error> {
    let processed = initialize_vector_store_with_notes(vector_store, model, notes).await?;
    info!("Initialized vector store with {} notes", processed);

    Ok(())
}
