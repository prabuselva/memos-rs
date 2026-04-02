use memos_rs::config::Config;
use memos_rs::vector::store::VectorStore;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = Config::default();
    
    println!("=== Qdrant API Integration Test ===\n");
    println!("Connected to Qdrant at: {}", config.vector.url);
    
    // Initialize vector store
    let vector_store = VectorStore::new(&config.vector.url).await?;
    
    println!("\n=== Qdrant Collection Info ===");
    check_qdrant_collections(&config.vector.url).await;
    
    // Show payload structure
    println!("\n=== Sample Payload Structure ===");
    show_sample_payload(&config.vector.url).await;
    
    // Test with existing data
    println!("\n=== Test Query (no filter) ===");
    test_query(&vector_store, &config.vector.url).await;
    
    Ok(())
}

async fn check_qdrant_collections(url: &str) {
    let client = reqwest::Client::new();
    let collections_url = format!("{}/collections", url);
    
    match client.get(&collections_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let body = response.text().await.unwrap_or_default();
                println!("Collections response:\n{}", body);
                
                // Try to parse and show collection names
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(collections) = json.get("result").and_then(|r| r.get("collections")) {
                        if let Some(collections_array) = collections.as_array() {
                            println!("\nCollection names:");
                            for collection in collections_array {
                                if let Some(name) = collection.get("name").and_then(|n| n.as_str()) {
                                    println!("  - {}", name);
                                    
                                    // Get collection info
                                    let collection_url = format!("{}/collections/{}", url, name);
                                    if let Ok(col_response) = client.get(&collection_url).send().await {
                                        if col_response.status().is_success() {
                                            if let Ok(col_body) = col_response.text().await {
                                                println!("    Info: {}", col_body);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                println!("Failed to get collections: {:?}", response.status());
            }
        }
        Err(e) => {
            println!("Error connecting to Qdrant: {}", e);
            println!("Make sure Qdrant is running at {}", url);
        }
    }
}

async fn show_sample_payload(url: &str) {
    let client = reqwest::Client::new();
    let search_url = format!("{}/collections/notes/points/search", url);
    
    let request = serde_json::json!({
        "vector": vec![0.1f32; 384],
        "limit": 1,
        "with_payload": true,
        "with_vector": false
    });
    
    match client.post(&search_url).json(&request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(results) = json.get("result").and_then(|r| r.as_array()) {
                        if let Some(first_point) = results.get(0) {
                            println!("Sample point structure:");
                            println!("  ID: {}", first_point.get("id").and_then(|v| v.as_str()).unwrap_or("N/A"));
                            println!("  Score: {}", first_point.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0));
                            
                            if let Some(payload) = first_point.get("payload") {
                                println!("  Payload keys: {:?}", payload.as_object().map(|o| o.keys().map(|k| k.as_str()).collect::<Vec<_>>()));
                                
                                if let Some(user_id) = payload.get("user_id").and_then(|v| v.as_str()) {
                                    println!("  User ID: {}", user_id);
                                }
                                
                                if let Some(title) = payload.get("title").and_then(|v| v.as_str()) {
                                    println!("  Title: {}", title);
                                }
                                
                                if let Some(content) = payload.get("content").and_then(|v| v.as_str()) {
                                    println!("  Content (first 100 chars): {}", &content[..content.chars().take(100).count()]);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

async fn test_query(vector_store: &VectorStore, url: &str) {
    let client = reqwest::Client::new();
    let search_url = format!("{}/collections/notes/points/search", url);
    
    // Test 1: No filter (all points)
    println!("\nQuery 1: No filter (top 3 points)");
    let request = serde_json::json!({
        "vector": vec![0.1f32; 384],
        "limit": 3,
        "with_payload": true,
        "with_vector": false
    });
    
    match client.post(&search_url).json(&request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(results) = json.get("result").and_then(|r| r.as_array()) {
                        println!("Found {} points (no filter)", results.len());
                        for (i, point) in results.iter().enumerate() {
                            if let Some(payload) = point.get("payload") {
                                if let Some(title) = payload.get("title").and_then(|v| v.as_str()) {
                                    if let Some(uid) = payload.get("user_id").and_then(|v| v.as_str()) {
                                        println!("  {}. Score: {:.3} - [User: {}] {}", i + 1, point.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0), uid, title);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    
    // Test 2: With user filter (if we have any user_id)
    println!("\nQuery 2: With user_id filter");
    let user_id_to_test = "dfafbb5d-e242-4d9b-af67-5b33c68ad6fa"; // From first point
    let request = serde_json::json!({
        "vector": vec![0.1f32; 384],
        "limit": 3,
        "filter": {
            "must": [{
                "key": "user_id",
                "match": {
                    "value": user_id_to_test
                }
            }]
        },
        "with_payload": true,
        "with_vector": false
    });
    
    match client.post(&search_url).json(&request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(results) = json.get("result").and_then(|r| r.as_array()) {
                        println!("Found {} points (with filter)", results.len());
                        for (i, point) in results.iter().enumerate() {
                            if let Some(payload) = point.get("payload") {
                                if let Some(title) = payload.get("title").and_then(|v| v.as_str()) {
                                    println!("  {}. Score: {:.3} - {}", i + 1, point.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0), title);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
