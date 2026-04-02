use memos_rs::config::Config;
use memos_rs::embeddings::ModelDownloader;
use memos_rs::{call_llm_api, BERTModel, Database, Note, NoteReference};

fn note_to_reference(note: &Note) -> NoteReference {
    NoteReference {
        id: 0,
        note_id: "test-note-id".to_string(),
        title: note.title.clone(),
        content: note.content.clone(),
        score: 1.0,
        distance: 0.0,
        user_id: None,
        created_at: None,
        updated_at: None,
        tags: vec![],
    }
}

#[tokio::test]
#[ignore]
async fn test_chat_mode_with_context() -> Result<(), anyhow::Error> {
    let config = Config::default();
    let downloader = ModelDownloader::new(&config.vector.model_cache_dir);
    downloader.ensure_model_downloaded()?;

    let tokenizer_path = config.get_tokenizer_path();
    let model_dir = tokenizer_path.parent().unwrap();
    let _model = BERTModel::from_tokenizer(
        &tokenizer_path.to_string_lossy(),
        &model_dir.to_string_lossy(),
    )?;

    let db = Database::new(&config).await?;

    let context_notes: Vec<Note> = vec![Note::new(
        "Rust Basics".to_string(),
        "Rust is a systems programming language. It has ownership model for memory safety."
            .to_string(),
    )
    .with_user_id("test-user".to_string())];

    for note in &context_notes {
        db.create_note_with_user(note.clone()).await?;
    }

    let note_ids: Vec<String> = context_notes.iter().map(|n| n.id.clone()).collect();
    let context_notes_from_db = db.get_notes_by_ids(&note_ids).await?;
    let context_refs: Vec<NoteReference> = context_notes_from_db
        .iter()
        .map(note_to_reference)
        .collect();
    let user_profile = memos_rs::models::UserProfile {
        id: "test-user".to_string(),
        username: "test".to_string(),
        email: "test@example.com".to_string(),
        created_at: chrono::Utc::now(),
        search_mode: "sql".to_string(),
        llm_settings: serde_json::json!({
            "provider": "openai",
            "url": "http://localhost:11434/v1",
            "api_key": serde_json::Value::Null,
            "model": "llama3",
            "temperature": 0.7,
            "max_tokens": 2048
        }),
    };

    let response = call_llm_api("What is Rust?", &context_refs, &user_profile)
        .await
        .map_err(|e| anyhow::anyhow!(e.1))?;

    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.contains("Rust"), "Response should mention Rust");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_chat_without_context() -> Result<(), anyhow::Error> {
    let context_notes: Vec<NoteReference> = vec![];

    let user_profile = memos_rs::models::UserProfile {
        id: "test-user".to_string(),
        username: "test".to_string(),
        email: "test@example.com".to_string(),
        created_at: chrono::Utc::now(),
        search_mode: "sql".to_string(),
        llm_settings: serde_json::json!({
            "provider": "openai",
            "url": "http://localhost:11434/v1",
            "api_key": serde_json::Value::Null,
            "model": "llama3",
            "temperature": 0.7,
            "max_tokens": 2048
        }),
    };

    let response = call_llm_api("Hello, how are you?", &context_notes, &user_profile)
        .await
        .map_err(|e| anyhow::anyhow!(e.1))?;

    assert!(!response.is_empty(), "Response should not be empty");

    Ok(())
}
