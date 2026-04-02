use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

pub struct ModelDownloader {
    cache_dir: PathBuf,
}

impl ModelDownloader {
    pub fn new(cache_dir: &str) -> Self {
        Self {
            cache_dir: PathBuf::from(cache_dir),
        }
    }

    pub fn ensure_model_downloaded(&self) -> Result<()> {
        let model_dir = self.cache_dir.join("all-MiniLM-L6-v2");

        let tokenizer_path = model_dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            self.download_tokenizer()?;
        }

        let config_path = model_dir.join("config.json");
        if !config_path.exists() {
            self.download_config()?;
        }

        let weights_path = model_dir.join("pytorch_model.bin");
        if !weights_path.exists() {
            self.download_weights()?;
        }

        Ok(())
    }

    pub fn get_tokenizer_path(&self) -> PathBuf {
        self.cache_dir
            .join("all-MiniLM-L6-v2")
            .join("tokenizer.json")
    }

    fn download_tokenizer(&self) -> Result<()> {
        let model_dir = self.cache_dir.join("all-MiniLM-L6-v2");
        std::fs::create_dir_all(&model_dir)?;

        let url = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json";
        let output_path = model_dir.join("tokenizer.json");

        info!("Downloading tokenizer from: {}", url);
        info!("Saving to: {}", output_path.display());

        let response = reqwest::blocking::get(url)?;
        let content = response.bytes()?;

        std::fs::write(&output_path, &content)?;

        info!("Tokenizer downloaded successfully!");

        Ok(())
    }

    fn download_config(&self) -> Result<()> {
        let model_dir = self.cache_dir.join("all-MiniLM-L6-v2");
        std::fs::create_dir_all(&model_dir)?;

        let url = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json";
        let output_path = model_dir.join("config.json");

        info!("Downloading config from: {}", url);
        info!("Saving to: {}", output_path.display());

        let response = reqwest::blocking::get(url)?;
        let content = response.bytes()?;

        std::fs::write(&output_path, &content)?;

        info!("Config downloaded successfully!");

        Ok(())
    }

    fn download_weights(&self) -> Result<()> {
        let model_dir = self.cache_dir.join("all-MiniLM-L6-v2");
        std::fs::create_dir_all(&model_dir)?;

        let url = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/model.safetensors";
        let output_path = model_dir.join("model.safetensors");

        info!("Downloading model weights from: {}", url);
        info!("Saving to: {}", output_path.display());

        let response = reqwest::blocking::get(url)?;
        let content = response.bytes()?;

        std::fs::write(&output_path, &content)?;

        info!("Model weights downloaded successfully!");

        Ok(())
    }
}
