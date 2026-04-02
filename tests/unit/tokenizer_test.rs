use memos_rs::embeddings::tokenizer::EmbeddingTokenizer;
use memos_rs::Config;

#[test]
fn test_tokenizer_load() {
    let config = Config::default();
    let tokenizer_path = config.get_tokenizer_path();
    let tokenizer = EmbeddingTokenizer::load(tokenizer_path.to_str().unwrap());
    assert!(tokenizer.is_ok());
}

#[test]
fn test_tokenize_basic() {
    let config = Config::default();
    let tokenizer_path = config.get_tokenizer_path();
    let tokenizer = EmbeddingTokenizer::load(tokenizer_path.to_str().unwrap()).unwrap();
    let text = "test note content";
    let tokens = tokenizer.tokenize(&text).unwrap();

    assert!(!tokens.is_empty());
    assert!(tokens.len() <= tokenizer.get_max_length());
}

#[test]
fn test_tokenize_long_text() {
    let config = Config::default();
    let tokenizer_path = config.get_tokenizer_path();
    let tokenizer = EmbeddingTokenizer::load(tokenizer_path.to_str().unwrap()).unwrap();
    let text = "a".repeat(500);
    let tokens = tokenizer.tokenize(&text).unwrap();

    assert!(tokens.len() <= tokenizer.get_max_length());
}
