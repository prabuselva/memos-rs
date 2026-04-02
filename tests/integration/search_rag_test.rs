use memos_rs::config::Config;
use memos_rs::embeddings::ModelDownloader;
use memos_rs::vector::store::VectorStore;
use memos_rs::{
    generate_random_notes, initialize_vector_store_with_notes, BERTModel, Database, Note,
};
use uuid::Uuid;

#[tokio::test]
#[ignore]
async fn test_search_rag_full_flow() -> Result<(), anyhow::Error> {
    let config = Config::default();
    let downloader = ModelDownloader::new(&config.vector.model_cache_dir);
    downloader.ensure_model_downloaded()?;

    let tokenizer_path = config.get_tokenizer_path();
    let model_dir = tokenizer_path.parent().unwrap();
    let model = BERTModel::from_tokenizer(
        &tokenizer_path.to_string_lossy(),
        &model_dir.to_string_lossy(),
    )?;

    let db = Database::new(&config).await?;
    let vector_store = VectorStore::new(&config.vector.url).await?;

    // Create a test user first
    let user_id = Uuid::new_v4().to_string();
    let user = memos_rs::User::new(
        "test_user".to_string(),
        "test@example.com".to_string(),
        "$2b$12$test_hashed_password".to_string(), // dummy hash
    )
    .with_id(user_id.clone());

    db.create_user(user).await?;

    let test_notes = generate_random_notes(100, &user_id);
    for note in &test_notes {
        db.create_note_with_user(note.clone()).await?;
    }

    initialize_vector_store_with_notes(&vector_store, &model, &test_notes).await?;

    let query = "What is machine learning?";
    let embedding = model.embed(query)?;
    let results = db.search_notes_by_vector(&user_id, &embedding, 5).await?;

    assert!(results.len() > 0, "Should return search results");
    assert!(results.len() <= 5, "Should return at most 5 results");

    for result in &results {
        assert!(!result.title.is_empty(), "Result should have title");
        assert!(!result.content.is_empty(), "Result should have content");
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_rag_with_context() -> Result<(), anyhow::Error> {
    let config = Config::default();
    let downloader = ModelDownloader::new(&config.vector.model_cache_dir);
    downloader.ensure_model_downloaded()?;

    let tokenizer_path = config.get_tokenizer_path();
    let model_dir = tokenizer_path.parent().unwrap();
    let model = BERTModel::from_tokenizer(
        &tokenizer_path.to_string_lossy(),
        &model_dir.to_string_lossy(),
    )?;

    let db = Database::new(&config).await?;
    let vector_store = VectorStore::new(&config.vector.url).await?;

    // Create a test user first
    let user_id = Uuid::new_v4().to_string();
    let user = memos_rs::User::new(
        "test_user_2".to_string(),
        "test2@example.com".to_string(),
        "$2b$12$test_hashed_password2".to_string(), // dummy hash
    )
    .with_id(user_id.clone());

    db.create_user(user).await?;

    let notes: Vec<Note> = vec![
        Note::new(
            "What is Rust?".to_string(),
            "Rust is a systems programming language focused on safety, speed, and concurrency. It does not use a garbage collector.".to_string(),
        ).with_user_id(user_id.clone()),
        Note::new(
            "What is Python?".to_string(),
            "Python is a high-level, interpreted programming language known for its simple syntax and readability.".to_string(),
        ).with_user_id(user_id.clone()),
    ];

    for note in &notes {
        db.create_note_with_user(note.clone()).await?;
    }

    initialize_vector_store_with_notes(&vector_store, &model, &notes).await?;

    let query = "Rust programming language";
    let embedding = model.embed(query)?;
    let results = db.search_notes_by_vector(&user_id, &embedding, 5).await?;

    let rust_found = results.iter().any(|r| r.title.contains("Rust"));
    assert!(rust_found, "Should find Rust-related notes");

    Ok(())
}
