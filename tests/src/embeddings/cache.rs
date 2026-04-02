use anyhow::Result;
use sha2::{Digest, Sha256};
use std::clone::Clone;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct EmbeddingCache {
    directory: PathBuf,
}

impl EmbeddingCache {
    pub fn new(cache_dir: &str) -> Self {
        Self {
            directory: PathBuf::from(cache_dir),
        }
    }

    pub fn get(&self, text: &str) -> Option<Vec<f32>> {
        let hash = self.compute_text_hash(text);
        let cache_file = self.directory.join(&hash);

        if !cache_file.exists() {
            return None;
        }

        let content = fs::read_to_string(cache_file).ok()?;
        let embedding: Vec<f32> = serde_json::from_str(&content).ok()?;

        Some(embedding)
    }

    pub fn set(&self, text: &str, embedding: &[f32]) -> Result<()> {
        let hash = self.compute_text_hash(text);
        let cache_file = self.directory.join(&hash);

        let json = serde_json::to_string(embedding)?;
        fs::write(cache_file, json)?;

        Ok(())
    }

    fn compute_text_hash(&self, text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
