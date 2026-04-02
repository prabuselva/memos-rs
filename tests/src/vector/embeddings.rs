pub struct EmbeddingModel {
    model: crate::embeddings::BERTModel,
}

use tracing::info;

impl EmbeddingModel {
    pub fn load(config: &crate::config::Config) -> Result<Self, anyhow::Error> {
        let tokenizer_path = config.get_tokenizer_path();
        let model_dir = config.get_model_dir();

        let model = crate::embeddings::BERTModel::from_tokenizer(
            tokenizer_path.to_str().unwrap(),
            model_dir.to_str().unwrap(),
        )?;

        Ok(Self { model })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, anyhow::Error> {
        info!(
            "Generating embedding for text ({} chars) using BERT model",
            text.len()
        );

        self.model.embed(text)
    }
}
