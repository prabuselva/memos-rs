use crate::config::Config;
use crate::embeddings::bert::BERTModel;
use crate::embeddings::tokenizer::EmbeddingTokenizer;
use anyhow::Result;

pub trait EmbeddingModel: Send + Sync {
    fn name(&self) -> &str;
    fn dimension(&self) -> usize;
    fn embed(&self, _text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, _texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn supports_batching(&self) -> bool;
}

pub struct EmbeddingModelWithTokenizer {
    tokenizer: EmbeddingTokenizer,
}

impl EmbeddingModelWithTokenizer {
    pub fn load(config: &Config) -> Result<Self> {
        let tokenizer_path = config.get_tokenizer_path();
        let tokenizer = EmbeddingTokenizer::load(tokenizer_path.to_str().unwrap())?;
        Ok(Self { tokenizer })
    }

    pub fn load_from_path(tokenizer_path: &str) -> Result<Self> {
        let tokenizer = EmbeddingTokenizer::load(tokenizer_path)?;
        Ok(Self { tokenizer })
    }

    pub fn tokenize(&self, text: &str) -> Result<Vec<u32>> {
        self.tokenizer.tokenize(text)
    }
}

impl EmbeddingModel for EmbeddingModelWithTokenizer {
    fn name(&self) -> &str {
        "all-MiniLM-L6-v2"
    }

    fn dimension(&self) -> usize {
        384
    }

    fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        unimplemented!("Embedding generation requires model weights")
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }

    fn supports_batching(&self) -> bool {
        true
    }
}

impl BERTModel {
    pub fn from_tokenizer(tokenizer_path: &str, model_dir: &str) -> Result<BERTModel> {
        BERTModel::load(tokenizer_path, model_dir)
    }
}

impl EmbeddingModel for BERTModel {
    fn name(&self) -> &str {
        "all-MiniLM-L6-v2"
    }

    fn dimension(&self) -> usize {
        384
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embed(text)
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.embed_batch(texts)
    }

    fn supports_batching(&self) -> bool {
        true
    }
}
