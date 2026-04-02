use memos_rs::config::Config;
use memos_rs::embeddings::ModelDownloader;
use memos_rs::vector::store::VectorStore;
use memos_rs::{BERTModel, Database, Note};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = Config::default();
    
    println!("=== Testing User Isolation with abcuser ===\n");
    
    let downloader = ModelDownloader::new(&config.vector.model_cache_dir);
    downloader.ensure_model_downloaded()?;
    
    let tokenizer_path = config.get_tokenizer_path();
    let model_dir = tokenizer_path.parent().unwrap();
    let model = BERTModel::from_tokenizer(
        &tokenizer_path.to_string_lossy(),
        &model_dir.to_string_lossy(),
    )?;
    
    let db = Database::new(&config).await?;
    
    let user_id = "998d57da-8814-47e5-904d-c7708511e6e5";
    let username = "abcuser";
    
    println!("User: {} (ID: {})\n", username, user_id);
    
    println!("Step 1: Deleting all existing notes for user...");
    db.delete_user_notes(user_id).await?;
    println!("✓ Deleted all existing notes\n");
    
    println!("Step 2: Creating 10 sample notes...");
    let sample_notes = vec![
        ("Introduction to Rust", "Rust is a systems programming language focused on safety, speed, and concurrency. It does not use a garbage collector."),
        ("Understanding Borrowing", "Borrowing is a key concept in Rust that allows you to refer to data without taking ownership. The two types are mutable and immutable borrowing."),
        ("Vector Databases", "Vector databases store embeddings (vectors) for efficient similarity search. They are used in RAG applications and semantic search."),
        ("Machine Learning Basics", "Machine learning is a subset of artificial intelligence that enables systems to learn from data. Common algorithms include linear regression and decision trees."),
        ("Neural Networks", "Neural networks are computing systems inspired by biological neural networks. They form the basis of deep learning and are used for image recognition and NLP."),
        ("Language Models", "Language models like GPT and Llama predict the next word in a sequence. They are trained on vast amounts of text data and can generate human-like text."),
        ("Embeddings Explained", "Embeddings are vector representations of text where similar meanings have similar vectors. Models like all-MiniLM-L6-v2 generate 384-dimensional embeddings."),
        ("RAG Applications", "Retrieval-Augmented Generation (RAG) combines information retrieval with text generation. It's used to provide context-aware responses in LLM applications."),
        ("Data Science Tools", "Popular data science tools include Python, pandas, NumPy, and scikit-learn. For production, tools like Ray and Dask help with distributed computing."),
        ("Cloud Computing", "Cloud computing platforms like AWS, Azure, and Google Cloud provide scalable infrastructure. Key services include EC2, S3, Lambda, and Cloud Functions."),
    ];
    
    let mut created_notes = Vec::new();
    for (title, content) in &sample_notes {
        let note = Note::new(title.to_string(), content.to_string())
            .with_user_id(user_id.to_string());
        
        let saved = db.create_note_with_user(note).await?;
        created_notes.push(saved);
        println!("  - Created: {}", title);
    }
    
    println!("\n✓ Created {} notes\n", created_notes.len());
    
    println!("Step 3: Verifying notes in vector store...");
    let vector_store = VectorStore::new(&config.vector.url).await?;
    
    let collections_url = format!("{}/collections", config.vector.url);
    let client = reqwest::Client::new();
    let response = client.get(&collections_url).send().await?;
    
    if response.status().is_success() {
        let body = response.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body)?;
        
        if let Some(collections) = json.get("result").and_then(|r| r.get("collections")) {
            if let Some(collections_array) = collections.as_array() {
                let expected_collection = format!("notes_{}", user_id);
                let found = collections_array.iter().any(|c| {
                    c.get("name").and_then(|n| n.as_str()) == Some(&expected_collection)
                });
                
                if found {
                    println!("  ✓ User-specific collection exists: {}", expected_collection);
                } else {
                    println!("  ✗ User-specific collection not found: {}", expected_collection);
                }
                
                println!("\n  All collections:");
                for collection in collections_array {
                    if let Some(name) = collection.get("name").and_then(|n| n.as_str()) {
                        println!("    - {}", name);
                    }
                }
            }
        }
    }
    
    println!("\nStep 4: Testing vector search...");
    let query = "What is Rust programming?";
    let embedding = model.embed(query)?;
    
    let results = vector_store
        .search_notes_with_scores(embedding, user_id, 10)
        .await?;
    
    println!("  Query: \"{}\"\n", query);
    println!("  Found {} results:", results.len());
    
    for (i, result) in results.iter().enumerate() {
        if let Some(ref payload) = result.payload {
            if let Some(title) = payload.get("title").and_then(|v| v.as_str()) {
                if let Some(content) = payload.get("content").and_then(|v| v.as_str()) {
                    println!("    {}. Score: {:.3} - {}", i + 1, result.score, title);
                    println!("       Content preview: {}", &content[..content.chars().take(80).count()]);
                }
            }
        }
    }
    
    println!("\nStep 5: Testing with different search queries...\n");
    
    let test_queries = vec![
        "machine learning algorithms",
        "neural network architecture",
        "vector embeddings",
    ];
    
    for test_query in &test_queries {
        let embedding = model.embed(test_query)?;
        let results = vector_store.search_notes_with_scores(embedding, user_id, 5).await?;
        
        println!("  Query: \"{}\"", test_query);
        println!("    Top result:");
        if let Some(first) = results.first() {
            if let Some(ref payload) = first.payload {
                if let Some(title) = payload.get("title").and_then(|v| v.as_str()) {
                    println!("      - {} (score: {:.3})", title, first.score);
                }
            }
        }
        println!();
    }
    
    println!("Step 6: Testing delete note...");
    if let Some(note) = created_notes.first() {
        let note_id = &note.id;
        db.delete(note_id).await?;
        println!("  - Deleted note: {}", note.title);
        
        let embedding_for_delete_check = model.embed("test")?;
        let results = vector_store
            .search_notes_with_scores(embedding_for_delete_check, user_id, 5)
            .await?;
        
        let deleted_still_exists = results.iter().any(|r| {
            r.payload.as_ref()
                .and_then(|p| p.get("id").and_then(|v| v.as_str()))
                == Some(note_id.as_str())
        });
        
        if deleted_still_exists {
            println!("  ✗ Note still exists in vector store");
        } else {
            println!("  ✓ Note successfully deleted from vector store");
        }
    }
    
    println!("\n=== All tests completed successfully! ===");
    
    Ok(())
}
