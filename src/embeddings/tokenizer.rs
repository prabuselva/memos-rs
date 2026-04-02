use anyhow::Result;

use tokenizers::Tokenizer;

#[derive(Clone)]
pub struct EmbeddingTokenizer {
    tokenizer: Tokenizer,
}

impl EmbeddingTokenizer {
    pub fn load(path: &str) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        Ok(Self { tokenizer })
    }

    pub fn tokenize(&self, text: &str) -> Result<Vec<u32>> {
        let encoding = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| anyhow::anyhow!("Failed to encode text: {}", e))?;

        let token_ids: Vec<u32> = encoding.get_ids().to_vec();

        Ok(token_ids)
    }

    pub fn get_max_length(&self) -> usize {
        512
    }

    pub fn pad_token_id(&self) -> Option<u32> {
        None
    }
}
