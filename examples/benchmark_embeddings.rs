use std::time::Instant;

fn get_model() -> memos_rs::embeddings::bert::BERTModel {
    let config = memos_rs::config::Config::default();
    let model_dir = config.get_model_dir();
    let model_dir_str = model_dir.to_str().unwrap();
    let tokenizer_path = config.get_tokenizer_path();
    let tokenizer_path_str = tokenizer_path.to_str().unwrap();
    memos_rs::embeddings::bert::BERTModel::load(tokenizer_path_str, model_dir_str)
        .expect("Failed to load model")
}

fn main() {
    println!("\n=== BERT Embedding Performance Benchmark ===\n");

    let model = get_model();

    let test_cases = vec![
        ("Short (5 tokens)", "Hello world"),
        ("Medium (20 tokens)", "This is a medium length text with some additional words to test the embedding performance."),
        ("Long (50 tokens)", "This is a longer text for testing purposes. It contains multiple sentences and various words. The quick brown fox jumps over the lazy dog. Programming is fun and challenging. Learning Rust is rewarding. BERT models are powerful for NLP tasks."),
        ("Very Long (100+ tokens)", "This is a very long text to test the performance with extended content. BERT embeddings are computed through multiple transformer layers. Each layer processes the sequence through self-attention and feed-forward networks. The model uses layer normalization and residual connections. GELU activation functions are applied in the intermediate layers. Position embeddings help the model understand token positions. Token type embeddings differentiate between sentences. The attention mechanism allows tokens to attend to each other. Self-attention computes query, key, and value projections. Softmax is used to compute attention probabilities. The context vector is a weighted sum of values. Layer normalization stabilizes training. Residual connections help with gradient flow. Feed-forward networks process each position independently. The final embedding is a 384-dimensional vector. Pooling extracts the [CLS] token representation. Tanh activation produces the final output."),
    ];

    println!("Single text embedding benchmarks:");
    println!("----------------------------------");
    for (name, text) in test_cases {
        let start = Instant::now();
        let embedding = model.embed(text).expect("Failed to embed");
        let duration = start.elapsed();

        println!(
            "{:<25} {:>10.4?} ({} dims)",
            name,
            duration,
            embedding.len()
        );
    }

    println!("\nBatch embedding benchmarks:");
    println!("---------------------------");

    let batches = vec![
        ("Batch 3 short", vec!["Text 1", "Text 2", "Text 3"]),
        ("Batch 5 medium", vec![
            "Medium text one with some content.",
            "Medium text two with different content.",
            "Medium text three with more content.",
            "Medium text four with additional content.",
            "Medium text five with extra content.",
        ]),
        ("Batch 3 long", vec![
            "Long text one with substantial content. BERT models process text through transformer layers. Each layer has self-attention and feed-forward networks.",
            "Long text two with different content. Layer normalization helps with training stability. Residual connections prevent gradient vanishing.",
            "Long text three with more content. Feed-forward networks process each position independently. GELU activation is used in intermediate layers.",
        ]),
    ];

    for (name, texts) in batches {
        let start = Instant::now();
        let embeddings = model.embed_batch(&texts).expect("Failed to batch embed");
        let duration = start.elapsed();

        println!(
            "{:<25} {:>10.4?} ({} texts, {} total dims)",
            name,
            duration,
            texts.len(),
            embeddings.len() * 384
        );
    }

    println!("\n=== Benchmark Complete ===\n");
}
