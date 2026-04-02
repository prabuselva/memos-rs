use std::path::Path;

#[test]
#[ignore]
fn test_load_model() {
    let cache_dir = ".memos-rs/models";
    let model_dir = format!("{}/all-MiniLM-L6-v2", cache_dir);

    if !Path::new(&model_dir).exists() {
        println!("Model directory not found, skipping test");
        return;
    }

    let model = memos_rs::embeddings::bert::BERTModel::load(&model_dir, &model_dir);

    assert!(model.is_ok(), "Failed to load model");
}

#[test]
#[ignore]
fn test_embed_text() {
    let cache_dir = ".memos-rs/models";
    let model_dir = format!("{}/all-MiniLM-L6-v2", cache_dir);

    if !Path::new(&model_dir).exists() {
        println!("Model directory not found, skipping test");
        return;
    }

    let model = memos_rs::embeddings::bert::BERTModel::load(&model_dir, &model_dir)
        .expect("Failed to load model");

    let text = "Hello, world!";
    let embedding = model.embed(text);

    assert!(embedding.is_ok(), "Failed to embed text");

    let embedding = embedding.unwrap();
    assert_eq!(embedding.len(), 384, "Embedding should have dimension 384");

    for val in &embedding {
        assert!(val.is_finite(), "Embedding value should be finite");
    }
}

#[test]
#[ignore]
fn test_embed_batch() {
    let cache_dir = ".memos-rs/models";
    let model_dir = format!("{}/all-MiniLM-L6-v2", cache_dir);

    if !Path::new(&model_dir).exists() {
        println!("Model directory not found, skipping test");
        return;
    }

    let model = memos_rs::embeddings::bert::BERTModel::load(&model_dir, &model_dir)
        .expect("Failed to load model");

    let texts = ["Hello, world!", "This is a test", "Another text"];
    let embeddings = model.embed_batch(&texts);

    assert!(embeddings.is_ok(), "Failed to embed batch");

    let embeddings = embeddings.unwrap();
    assert_eq!(embeddings.len(), 3, "Should have 3 embeddings");

    for embedding in &embeddings {
        assert_eq!(
            embedding.len(),
            384,
            "Each embedding should have dimension 384"
        );
    }
}

#[test]
fn test_tokenization() {
    let cache_dir = ".memos-rs/models";
    let model_dir = format!("{}/all-MiniLM-L6-v2", cache_dir);
    let tokenizer_path = format!("{}/tokenizer.json", model_dir);

    if !Path::new(&model_dir).exists() {
        println!("Model directory not found, skipping test");
        return;
    }

    if !Path::new(&tokenizer_path).exists() {
        println!("Tokenizer not found, skipping test");
        return;
    }

    let model = memos_rs::embeddings::bert::BERTModel::load(&tokenizer_path, &model_dir)
        .expect("Failed to load model");

    let text = "Hello, how are you?";
    let tokens = model.tokenize(text);

    assert!(tokens.is_ok(), "Failed to tokenize");
    let tokens = tokens.unwrap();

    assert!(!tokens.is_empty(), "Should have at least one token");
    assert!(tokens.len() <= 512, "Tokens should not exceed max length");
}
