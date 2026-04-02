pub use providers::{AnthropicProvider, GroqProvider, LLMProvider, OllamaProvider, OpenAIProvider};

mod providers;

use crate::models::auth_dto::LLMSettings;
use anyhow::Result;

impl LLMSettings {
    pub fn create_provider(&self) -> Result<LLMProvider> {
        let temperature = self.temperature as f32;

        match self.provider.as_str() {
            "ollama" => Ok(LLMProvider::Ollama(providers::OllamaProvider::new(
                self.url.clone(),
                self.model.clone(),
                temperature,
            ))),
            "openai" => {
                let api_key = self
                    .api_key
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("OpenAI API key is required"))?;
                Ok(LLMProvider::OpenAI(providers::OpenAIProvider::new(
                    api_key,
                    self.model.clone(),
                    self.url.clone(),
                    temperature,
                )))
            }
            "anthropic" => {
                let api_key = self
                    .api_key
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Anthropic API key is required"))?;
                Ok(LLMProvider::Anthropic(providers::AnthropicProvider::new(
                    api_key,
                    self.model.clone(),
                    temperature,
                )))
            }
            "groq" => {
                let api_key = self
                    .api_key
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Groq API key is required"))?;
                Ok(LLMProvider::Groq(providers::GroqProvider::new(
                    api_key,
                    self.model.clone(),
                    temperature,
                )))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported LLM provider: {}",
                self.provider
            )),
        }
    }
}

pub async fn test_llm_connection(settings: &LLMSettings) -> Result<bool> {
    let provider = settings.create_provider()?;
    provider.test_connection().await
}

pub async fn generate_llm_response(
    settings: &LLMSettings,
    query: &str,
    context: &[String],
) -> Result<String> {
    let provider = settings.create_provider()?;
    provider.generate_response(query, context).await
}

pub async fn generate_llm_response_with_history(
    settings: &LLMSettings,
    query: &str,
    context: &[String],
    history: Option<Vec<crate::api::chat::ChatMessageHistory>>,
) -> Result<String> {
    let provider = settings.create_provider()?;

    let messages = provider.build_messages_with_history(query, context, history)?;
    provider.generate_response_with_messages(&messages).await
}
