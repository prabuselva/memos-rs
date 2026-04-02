use memos_rs::config::Config;
use memos_rs::embeddings::ModelDownloader;
use memos_rs::vector::store::VectorStore;
use memos_rs::{generate_random_notes, initialize_vector_store_with_notes, BERTModel, Database, Note};
use uuid::Uuid;

#[tokio::test]
#[ignore]
async fn test_user_isolation_separate_collections() -> Result<(), anyhow::Error> {
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
    
    // Create two test users
    let user1_id = Uuid::new_v4().to_string();
    let user1 = memos_rs::User::new(
        "test_user1".to_string(),
        "user1@test.com".to_string(),
        "$2b$12$test_hashed_password".to_string(),
    )
    .with_id(user1_id.clone());
    
    db.create_user(user1).await?;
    
    let user2_id = Uuid::new_v4().to_string();
    let user2 = memos_rs::User::new(
        "test_user2".to_string(),
        "user2@test.com".to_string(),
        "$2b$12$test_hashed_password2".to_string(),
    )
    .with_id(user2_id.clone());
    
    db.create_user(user2).await?;
    
    info!("Created test users: {} and {}", user1_id, user2_id);
    
    // Create notes for user1
    let user1_notes = generate_random_notes(5, &user1_id);
    for note in &user1_notes {
        db.create_note_with_user(note.clone()).await?;
    }
    
    info!("Created {} notes for user1", user1_notes.len());
    
    // Create notes for user2
    let user2_notes = generate_random_notes(5, &user2_id);
    for note in &user2_notes {
        db.create_note_with_user(note.clone()).await?;
    }
    
    info!("Created {} notes for user2", user2_notes.len());
    
    // Initialize vector store with both users' notes
    initialize_vector_store_with_notes(&vector_store, &model, &user1_notes).await?;
    initialize_vector_store_with_notes(&vector_store, &model, &user2_notes).await?;
    
    info!("Initialized vector store with notes for both users");
    
    // Verify collections were created
    let collections_url = format!("{}/collections", config.vector.url);
    let client = reqwest::Client::new();
    let response = client.get(&collections_url).send().await?;
    
    if response.status().is_success() {
        let body = response.text().await?;
        info!("Collections: {}", body);
        
        let json: serde_json::Value = serde_json::from_str(&body)?;
        if let Some(collections) = json.get("result").and_then(|r| r.get("collections")) {
            if let Some(collections_array) = collections.as_array() {
                let user1_collection = format!("notes_{}", user1_id);
                let user2_collection = format!("notes_{}", user2_id);
                
                let user1_found = collections_array.iter().any(|c| {
                    c.get("name").and_then(|n| n.as_str()) == Some(&user1_collection)
                });
                let user2_found = collections_array.iter().any(|c| {
                    c.get("name").and_then(|n| n.as_str()) == Some(&user2_collection)
                });
                
                assert!(user1_found, "User1 collection should exist: {}", user1_collection);
                assert!(user2_found, "User2 collection should exist: {}", user2_collection);
                
                info!("✓ Both user-specific collections created successfully");
            }
        }
    }
    
    // Test search isolation - search for user1
    let query = "What is machine learning?";
    let embedding = model.embed(query)?;
    
    let user1_results = vector_store
        .search_notes_with_scores(embedding.clone(), &user1_id, 10)
        .await?;
    
    info!("User1 search results: {}", user1_results.len());
    assert_eq!(user1_results.len(), 5, "User1 should have 5 results");
    
    for result in &user1_results {
        if let Some(ref payload) = result.payload {
            if let Some(uid) = payload.get("user_id").and_then(|v| v.as_str()) {
                assert_eq!(uid, &user1_id, "All user1 results should have user1's user_id");
            }
        }
    }
    
    // Test search isolation - search for user2
    let user2_results = vector_store
        .search_notes_with_scores(embedding.clone(), &user2_id, 10)
        .await?;
    
    info!("User2 search results: {}", user2_results.len());
    assert_eq!(user2_results.len(), 5, "User2 should have 5 results");
    
    for result in &user2_results {
        if let Some(ref payload) = result.payload {
            if let Some(uid) = payload.get("user_id").and_then(|v| v.as_str()) {
                assert_eq!(uid, &user2_id, "All user2 results should have user2's user_id");
            }
        }
    }
    
    info!("✓ User isolation verified - user1 and user2 searches are isolated");
    
    // Verify user1 results don't contain user2's notes
    let user1_has_user2_notes = user1_results.iter().any(|r| {
        r.payload.as_ref()
            .and_then(|p| p.get("user_id").and_then(|v| v.as_str()))
            == Some(&user2_id)
    });
    assert!(
        !user1_has_user2_notes,
        "User1 results should not contain user2's notes"
    );
    
    // Verify user2 results don't contain user1's notes
    let user2_has_user1_notes = user2_results.iter().any(|r| {
        r.payload.as_ref()
            .and_then(|p| p.get("user_id").and_then(|v| v.as_str()))
            == Some(&user1_id)
    });
    assert!(
        !user2_has_user1_notes,
        "User2 results should not contain user1's notes"
    );
    
    info!("✓ No cross-user contamination detected");
    
    // Test delete user data
    vector_store.delete_user_data(&user1_id).await?;
    
    info!("Deleted user1's vector data");
    
    // Verify user1's collection is deleted
    let response = client
        .get(&format!("{}/collections/notes_{}", config.vector.url, user1_id))
        .send()
        .await?;
    
    assert!(
        !response.status().is_success(),
        "User1's collection should be deleted"
    );
    
    info!("✓ User1's collection successfully deleted");
    
    // Verify user2's data still exists
    let user2_results_after = vector_store
        .search_notes_with_scores(embedding.clone(), &user2_id, 10)
        .await?;
    
    assert_eq!(user2_results_after.len(), 5, "User2's data should still exist");
    
    info!("✓ User2's data still accessible after user1 deletion");
    
    Ok(())
}
