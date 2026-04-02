use memos_rs::embeddings::bert::{
    AttentionOutputWeights, AttentionWeights, BERTModel, EmbeddingWeights, EncoderWeights,
    FeedForwardWeights, LayerNormWeights, ModelConfig, ModelWeights,
};

#[test]
fn test_model_config() {
    let config = ModelConfig {
        hidden_size: 384,
        num_hidden_layers: 6,
        num_attention_heads: 12,
        intermediate_size: 1536,
        max_position_embeddings: 512,
        vocab_size: 30522,
        hidden_act: "gelu".to_string(),
    };

    assert_eq!(config.hidden_size, 384);
    assert_eq!(config.num_hidden_layers, 6);
    assert_eq!(config.num_attention_heads, 12);
    assert_eq!(config.intermediate_size, 1536);
    assert_eq!(config.max_position_embeddings, 512);
    assert_eq!(config.vocab_size, 30522);
    assert_eq!(config.hidden_act, "gelu");
}

#[test]
fn test_layer_norm_weights() {
    let weights = LayerNormWeights {
        weight: vec![1.0; 384],
        bias: vec![0.0; 384],
    };

    assert_eq!(weights.weight.len(), 384);
    assert_eq!(weights.bias.len(), 384);
}

#[test]
fn test_attention_weights() {
    let attention = AttentionWeights {
        query: vec![0.0; 384 * 384],
        key: vec![0.0; 384 * 384],
        value: vec![0.0; 384 * 384],
        output: AttentionOutputWeights {
            dense: vec![0.0; 384 * 384],
            layer_norm: LayerNormWeights {
                weight: vec![1.0; 384],
                bias: vec![0.0; 384],
            },
        },
    };

    assert_eq!(attention.query.len(), 384 * 384);
    assert_eq!(attention.key.len(), 384 * 384);
    assert_eq!(attention.value.len(), 384 * 384);
}

#[test]
fn test_encoder_weights() {
    let encoder = EncoderWeights {
        attention: AttentionWeights {
            query: vec![0.0; 384 * 384],
            key: vec![0.0; 384 * 384],
            value: vec![0.0; 384 * 384],
            output: AttentionOutputWeights {
                dense: vec![0.0; 384 * 384],
                layer_norm: LayerNormWeights {
                    weight: vec![1.0; 384],
                    bias: vec![0.0; 384],
                },
            },
        },
        intermediate: FeedForwardWeights {
            dense: vec![0.0; 384 * 1536],
            layer_norm: LayerNormWeights {
                weight: vec![1.0; 1536],
                bias: vec![0.0; 1536],
            },
        },
        output: FeedForwardWeights {
            dense: vec![0.0; 384 * 1536],
            layer_norm: LayerNormWeights {
                weight: vec![1.0; 384],
                bias: vec![0.0; 384],
            },
        },
    };

    assert_eq!(encoder.intermediate.dense.len(), 384 * 1536);
    assert_eq!(encoder.output.dense.len(), 384 * 1536);
}

#[test]
fn test_model_weights() {
    let weights = ModelWeights {
        embeddings: EmbeddingWeights {
            word_embeddings: vec![0.0; 30522 * 384],
            position_embeddings: vec![0.0; 512 * 384],
            token_type_embeddings: vec![0.0; 2 * 384],
            layer_norm: LayerNormWeights {
                weight: vec![1.0; 384],
                bias: vec![0.0; 384],
            },
        },
        encoder: vec![],
        pooler: None,
    };

    assert_eq!(weights.embeddings.word_embeddings.len(), 30522 * 384);
    assert_eq!(weights.embeddings.position_embeddings.len(), 512 * 384);
}

#[test]
fn test_gelu_activation() {
    assert!((BERTModel::gelu(1.0) - 0.841192).abs() < 0.01);
    assert!((BERTModel::gelu(0.0) - 0.0).abs() < 0.001);
    assert!((BERTModel::gelu(-1.0) - (-0.158808)).abs() < 0.01);
}

#[test]
fn test_layer_norm_per_feature() {
    let input = vec![1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let weights = LayerNormWeights {
        weight: vec![1.0; 4],
        bias: vec![0.0; 4],
    };

    let output = layer_norm_forward_per_feature(&input, &weights);

    assert_eq!(output.len(), 8);

    for feat in 0..4 {
        let mean = (input[feat] + input[feat + 4]) / 2.0;
        let variance = ((input[feat] - mean).powf(2.0) + (input[feat + 4] - mean).powf(2.0)) / 2.0;
        let std = variance.sqrt();
        let epsilon = 1e-12;

        let expected_0 = (input[feat] - mean) / (std + epsilon);
        let expected_1 = (input[feat + 4] - mean) / (std + epsilon);

        assert!(
            (output[feat] - expected_0).abs() < 0.01,
            "Index {}: output={}, expected={}, mean={}, std={}",
            feat,
            output[feat],
            expected_0,
            mean,
            std
        );
        assert!(
            (output[feat + 4] - expected_1).abs() < 0.01,
            "Index {}: output={}, expected={}, mean={}, std={}",
            feat + 4,
            output[feat + 4],
            expected_1,
            mean,
            std
        );
    }
}

#[test]
fn test_add_residual() {
    let main = vec![1.0, 2.0, 3.0, 4.0];
    let residual = vec![0.1, 0.2, 0.3, 0.4];

    let result = add_residual(&main, &residual);

    assert_eq!(result.len(), 4);
    assert!((result[0] - 1.1).abs() < 0.001);
    assert!((result[1] - 2.2).abs() < 0.001);
    assert!((result[2] - 3.3).abs() < 0.001);
    assert!((result[3] - 4.4).abs() < 0.001);
}

fn layer_norm_forward_per_feature(input: &[f32], weights: &LayerNormWeights) -> Vec<f32> {
    let input_size = input.len();
    let feature_size = weights.weight.len();
    let epsilon = 1e-12;

    let sequence_length = input_size / feature_size;
    let mut output = vec![0.0; input_size];

    for feat in 0..feature_size {
        let mut sum = 0.0;
        for seq in 0..sequence_length {
            sum += input[seq * feature_size + feat];
        }
        let mean = sum / sequence_length as f32;

        let mut var_sum = 0.0;
        for seq in 0..sequence_length {
            let diff = input[seq * feature_size + feat] - mean;
            var_sum += diff * diff;
        }
        let variance = var_sum / sequence_length as f32;

        for seq in 0..sequence_length {
            let idx = seq * feature_size + feat;
            let normalized = (input[idx] - mean) / (variance + epsilon).sqrt();
            output[idx] = normalized * weights.weight[feat] + weights.bias[feat];
        }
    }

    output
}

fn add_residual(main: &[f32], residual: &[f32]) -> Vec<f32> {
    assert_eq!(main.len(), residual.len());

    let mut result = vec![0.0; main.len()];
    for i in 0..main.len() {
        result[i] = main[i] + residual[i];
    }
    result
}

fn get_model() -> BERTModel {
    let config = memos_rs::config::Config::default();
    let model_dir = config.get_model_dir();
    let model_dir_str = model_dir.to_str().unwrap();
    let tokenizer_path = config.get_tokenizer_path();
    let tokenizer_path_str = tokenizer_path.to_str().unwrap();
    memos_rs::embeddings::bert::BERTModel::load(tokenizer_path_str, model_dir_str)
        .expect("Failed to load model")
}

#[test]
fn test_short_text_embedding() {
    let model = get_model();
    let text = "Hello world";
    let embedding = model.embed(text).expect("Failed to embed");
    assert_eq!(embedding.len(), 384);
    assert!(!embedding.iter().any(|&x| x.is_nan()));
}

#[test]
fn test_medium_text_embedding() {
    let model = get_model();
    let text = "This is a medium length text with some additional words to test the embedding performance. It should work correctly without any issues.";
    let embedding = model.embed(text).expect("Failed to embed");
    assert_eq!(embedding.len(), 384);
    assert!(!embedding.iter().any(|&x| x.is_nan()));
}

#[test]
fn test_long_text_embedding() {
    let model = get_model();
    let text = "This is a longer text for testing purposes. It contains multiple sentences and various words. The quick brown fox jumps over the lazy dog. Programming is fun and challenging. Learning Rust is rewarding. BERT models are powerful for NLP tasks. Embeddings capture semantic meaning. Context matters in language understanding.";
    let embedding = model.embed(text).expect("Failed to embed");
    assert_eq!(embedding.len(), 384);
    assert!(!embedding.iter().any(|&x| x.is_nan()));
}

#[test]
fn test_very_long_text_embedding() {
    let model = get_model();
    let text = "This is a very long text to test the performance with extended content. BERT embeddings are computed through multiple transformer layers. Each layer processes the sequence through self-attention and feed-forward networks. The model uses layer normalization and residual connections. GELU activation functions are applied in the intermediate layers. Position embeddings help the model understand token positions. Token type embeddings differentiate between sentences. The attention mechanism allows tokens to attend to each other. Self-attention computes query, key, and value projections. Softmax is used to compute attention probabilities. The context vector is a weighted sum of values. Layer normalization stabilizes training. Residual connections help with gradient flow. Feed-forward networks process each position independently. The final embedding is a 384-dimensional vector. Pooling extracts the [CLS] token representation. Tanh activation produces the final output. This concludes the long text test.";
    let embedding = model.embed(text).expect("Failed to embed");
    assert_eq!(embedding.len(), 384);
    assert!(!embedding.iter().any(|&x| x.is_nan()));
}

#[test]
fn test_batch_embeddings() {
    let model = get_model();
    let texts = vec!["First short text", "Second short text", "Third short text"];
    let embeddings = model.embed_batch(&texts).expect("Failed to batch embed");
    assert_eq!(embeddings.len(), 3);
    for embedding in &embeddings {
        assert_eq!(embedding.len(), 384);
        assert!(!embedding.iter().any(|&x| x.is_nan()));
    }
}

#[test]
fn test_embedding_consistency() {
    let model = get_model();
    let text = "Consistency test text";
    let embedding1 = model.embed(text).expect("Failed to embed");
    let embedding2 = model.embed(text).expect("Failed to embed");
    assert_eq!(embedding1.len(), embedding2.len());
    for (a, b) in embedding1.iter().zip(embedding2.iter()) {
        assert!((a - b).abs() < 1e-6, "Embeddings should be identical");
    }
}

#[test]
fn test_different_text_lengths() {
    let model = get_model();
    let test_texts = vec![
        "A",
        "Short",
        "Medium length text",
        "This is a medium length text with some content for testing.",
        "This is a long text with many words to test the embedding functionality. It should handle longer sequences properly. BERT models can process up to 512 tokens. This text is getting longer and should still work fine. Adding more content to ensure we have enough words. The embedding dimension is 384. Each position gets a vector representation. The model uses self-attention mechanisms. Layer normalization helps with training stability. Residual connections prevent gradient vanishing. Feed-forward networks process each token. GELU activation is used in intermediate layers. Position embeddings encode order information. Token type embeddings distinguish segments. The final output is a fixed-size vector regardless of input length. Pooling extracts meaningful representations. Tanh squashes values to [-1, 1] range. This concludes the comprehensive test text.",
    ];

    for text in test_texts {
        let embedding = model.embed(text).expect("Failed to embed");
        assert_eq!(embedding.len(), 384);
        assert!(
            !embedding.iter().any(|&x| x.is_nan()),
            "Embedding contains NaN values for text: {}",
            text
        );
    }
}

#[test]
fn test_special_characters() {
    let model = get_model();
    let text = "Test with special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?";
    let embedding = model.embed(text).expect("Failed to embed");
    assert_eq!(embedding.len(), 384);
    assert!(!embedding.iter().any(|&x| x.is_nan()));
}

#[test]
fn test_unicode_text() {
    let model = get_model();
    let text = "Hello 世界 🌍 مرحبا";
    let embedding = model.embed(text).expect("Failed to embed");
    assert_eq!(embedding.len(), 384);
    assert!(!embedding.iter().any(|&x| x.is_nan()));
}

#[test]
fn test_empty_string() {
    let model = get_model();
    let text = "";
    let result = model.embed(text);
    assert!(result.is_err(), "Empty string should fail");
}

#[test]
fn test_very_long_text_truncation() {
    let model = get_model();
    let text = "word ".repeat(600);
    let embedding = model.embed(&text).expect("Failed to embed");
    assert_eq!(embedding.len(), 384);
    assert!(!embedding.iter().any(|&x| x.is_nan()));
}

#[test]
fn test_embedding_values_range() {
    let model = get_model();
    let text = "Test text for value range checking";
    let embedding = model.embed(text).expect("Failed to embed");

    let min_val = embedding.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = embedding.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    assert!(
        min_val >= -1.0 && max_val <= 1.0,
        "Embedding values should be in [-1, 1] range due to tanh pooling. Got [{}, {}]",
        min_val,
        max_val
    );
}

#[test]
fn test_embedding_norm() {
    let model = get_model();
    let text = "Test text for embedding norm";
    let embedding = model.embed(text).expect("Failed to embed");

    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

    assert!(norm > 0.0, "Embedding norm should be positive");
    assert!(norm < 10.0, "Embedding norm should be reasonable");
}

#[test]
fn test_multiple_sequential_embeddings() {
    let model = get_model();
    let texts = vec![
        "First text",
        "Second text",
        "Third text",
        "Fourth text",
        "Fifth text",
    ];

    for text in texts {
        let embedding = model.embed(text).expect("Failed to embed");
        assert_eq!(embedding.len(), 384);
    }
}

#[test]
fn test_embedding_determinism() {
    let model = get_model();
    let text = "Determinism test";

    let embeddings: Vec<_> = (0..5)
        .map(|_| model.embed(text).expect("Failed to embed"))
        .collect();

    for i in 1..embeddings.len() {
        assert_eq!(embeddings[0].len(), embeddings[i].len());
        for (a, b) in embeddings[0].iter().zip(embeddings[i].iter()) {
            assert!(
                (a - b).abs() < 1e-6,
                "Embeddings should be identical across runs"
            );
        }
    }
}

#[test]
fn test_embedding_generation() {
    let config = memos_rs::config::Config::default();
    let model_dir = config.get_model_dir();
    let model_dir_str = model_dir.to_str().unwrap();
    let tokenizer_path = config.get_tokenizer_path();
    let tokenizer_path_str = tokenizer_path.to_str().unwrap();
    let model = memos_rs::embeddings::bert::BERTModel::load(tokenizer_path_str, model_dir_str)
        .expect("Failed to load model");

    let text = "test embedding";
    let embedding = model.embed(text).expect("Failed to generate embedding");

    eprintln!("Generated embedding with {} dimensions", embedding.len());
    assert_eq!(embedding.len(), 384);
}

#[cfg(feature = "optimize")]
mod manual_benchmark {
    use super::BERTModel;
    use std::time::Instant;

    fn get_model() -> BERTModel {
        let config = memos_rs::config::Config::default();
        let model_dir = config.get_model_dir();
        let model_dir_str = model_dir.to_str().unwrap();
        let tokenizer_path = config.get_tokenizer_path();
        let tokenizer_path_str = tokenizer_path.to_str().unwrap();
        memos_rs::embeddings::bert::BERTModel::load(tokenizer_path_str, model_dir_str)
            .expect("Failed to load model")
    }

    pub fn run_benchmark() {
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
}

#[test]
fn test_print_direct() {
    eprintln!("[TEST] Direct print test");
    assert!(true);
}

#[test]
fn test_semantic_similarity() {
    let model = get_model();

    let rust_embedding = model
        .embed("Rust programming language")
        .expect("Failed to embed");
    let python_embedding = model
        .embed("Python programming language")
        .expect("Failed to embed");
    let cat_embedding = model.embed("feline animal").expect("Failed to embed");

    let rust_norm: f32 = rust_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    let python_norm: f32 = python_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    let cat_norm: f32 = cat_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

    let rust_normed: Vec<f32> = rust_embedding.iter().map(|x| x / rust_norm).collect();
    let python_normed: Vec<f32> = python_embedding.iter().map(|x| x / python_norm).collect();
    let cat_normed: Vec<f32> = cat_embedding.iter().map(|x| x / cat_norm).collect();

    let rust_python: f32 = rust_normed
        .iter()
        .zip(python_normed.iter())
        .map(|(a, b)| a * b)
        .sum();

    let rust_cat: f32 = rust_normed
        .iter()
        .zip(cat_normed.iter())
        .map(|(a, b)| a * b)
        .sum();

    assert!(
        rust_python > rust_cat,
        "Rust and Python (both programming languages) should be more similar than Rust and cat"
    );
}
