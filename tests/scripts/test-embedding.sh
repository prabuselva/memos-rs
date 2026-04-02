#!/bin/bash

echo "=== Test Embedding Generation ==="
echo ""

# Create test binary
cat > /tmp/test_embedding.rs << 'EOF'
use std::path::Path;

fn main() {
    println!("Testing embedding model loading...\n");
    
    let model_path = std::env::args().nth(1).unwrap_or_else(|| "models/model.onnx".to_string());
    let model_path = Path::new(&model_path);
    
    if !model_path.exists() {
        println!("ERROR: Model not found at: {}", model_path.display());
        println!("Run setup script first: bash scripts/setup-embeddings.sh");
        std::process::exit(1);
    }
    
    println!("✓ Model file found: {}", model_path.display());
    
    // Test loading model
    println!("\nLoading embedding model...");
    
    let embedding_model = match load_embedding_model(&model_path) {
        Ok(model) => model,
        Err(e) => {
            println!("ERROR: Failed to load model: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("✓ Model loaded successfully\n");
    
    // Test embedding generation
    let test_texts = vec![
        "This is a test note about Rust programming",
        "Another note about machine learning and AI",
        "Quick brown fox jumps over the lazy dog",
    ];
    
    println!("Testing embedding generation:\n");
    
    for (i, text) in test_texts.iter().enumerate() {
        println!("Test {}: \"{}\"", i + 1, text);
        
        match embedding_model.embed(text) {
            Ok(embedding) => {
                println!("  ✓ Embedding generated");
                println!("  Dimension: {}", embedding.len());
                println!("  First 5 values: {:?}", &embedding[0..5]);
                println!("  Vector norm: {}", calculate_norm(&embedding));
            }
            Err(e) => {
                println!("  ERROR: {}", e);
            }
        }
        println!();
    }
    
    println!("All tests completed!");
}

fn load_embedding_model(model_path: &std::path::Path) -> Result<EmbeddingModel, Box<dyn std::error::Error>> {
    use ort::{Session, providers};
    use tokenizers::Tokenizer;
    
    let session = Session::builder()?
        .with_model_from_file(model_path)?
        .with_execution_providers(&[providers::Cpu::default()])?
        .build()?;
    
    let tokenizer_path = model_path.parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("tokenizer.json");
    
    let tokenizer = if tokenizer_path.exists() {
        Tokenizer::from_file(tokenizer_path)?
    } else {
        Tokenizer::from_pretrained("sentence-transformers/all-MiniLM-L6-v2", None)?
    };
    
    Ok(EmbeddingModel { session, tokenizer })
}

struct EmbeddingModel {
    session: Session,
    tokenizer: Tokenizer,
}

impl EmbeddingModel {
    fn embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let encoding = self.tokenizer.encode(text, true)?;
        
        let input_ids: Vec<i64> = encoding.get_ids().to_vec();
        let attention_mask: Vec<i64> = encoding.get_attention_mask().to_vec();
        let token_type_ids: Vec<i64> = encoding.get_type_ids().to_vec();
        
        let input_ids_tensor = ort::Tensor::from_array(&input_ids)?;
        let attention_mask_tensor = ort::Tensor::from_array(&attention_mask)?;
        let token_type_ids_tensor = ort::Tensor::from_array(&token_type_ids)?;
        
        let inputs = [
            ("input_ids", input_ids_tensor),
            ("attention_mask", attention_mask_tensor),
            ("token_type_ids", token_type_ids_tensor),
        ];
        
        let outputs = self.session.run(inputs)?;
        
        let output_tensor = outputs.get(0).ok_or("No output from model")?;
        
        let embeddings: Vec<f32> = output_tensor
            .try_extract_tensor::<f32>()?
            .to_vec();
        
        Ok(embeddings)
    }
}

fn calculate_norm(vector: &[f32]) -> f32 {
    vector.iter().map(|x| x * x).sum::<f32>().sqrt()
}
EOF

echo "Compiling test binary..."
rustc --edition 2021 /tmp/test_embedding.rs \
    -L /home/praburaja/projects/opencode_ws/memos-rs/target/release/deps \
    --extern ort=/home/praburaja/projects/opencode_ws/memos-rs/target/release/deps/libort-*.rlib \
    --extern tokenizers=/home/praburaja/projects/opencode_ws/memos-rs/target/release/deps/libtokenizers-*.rlib \
    -o /tmp/test_embedding 2>&1 | head -20

if [ ! -f /tmp/test_embedding ]; then
    echo "Compilation failed. Building the project first..."
    echo "Running: cargo build --release"
    cd /home/praburaja/projects/opencode_ws/memos-rs
    cargo build --release 2>&1 | tail -20
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "Build successful! Now running tests..."
    else
        echo ""
        echo "Build failed. Please check errors above."
        exit 1
    fi
else
    echo ""
    echo "Test binary compiled. Running..."
    /tmp/test_embedding
fi
