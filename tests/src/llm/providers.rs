use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

use crate::api::chat::ChatMessageHistory;

#[derive(Debug, Clone)]
pub enum LLMProvider {
    Ollama(OllamaProvider),
    OpenAI(OpenAIProvider),
    Anthropic(AnthropicProvider),
    Groq(GroqProvider),
}

impl LLMProvider {
    pub fn name(&self) -> &str {
        match self {
            LLMProvider::Ollama(_) => "ollama",
            LLMProvider::OpenAI(_) => "openai",
            LLMProvider::Anthropic(_) => "anthropic",
            LLMProvider::Groq(_) => "groq",
        }
    }

    pub async fn generate_response(&self, prompt: &str, context: &[String]) -> Result<String> {
        let full_prompt = self.build_prompt(prompt, context);
        match self {
            LLMProvider::Ollama(p) => p.generate_response(&full_prompt).await,
            LLMProvider::OpenAI(p) => p.generate_response(&full_prompt).await,
            LLMProvider::Anthropic(p) => p.generate_response(&full_prompt).await,
            LLMProvider::Groq(p) => p.generate_response(&full_prompt).await,
        }
    }

    pub fn build_messages_with_history(
        &self,
        query: &str,
        context: &[String],
        history: Option<Vec<ChatMessageHistory>>,
    ) -> Result<Vec<ChatMessage>> {
        let mut messages = Vec::new();

        if let Some(hist) = history {
            for msg in hist {
                messages.push(ChatMessage {
                    role: msg.role,
                    content: msg.content,
                });
            }
        }

        let context_text = context
            .iter()
            .enumerate()
            .map(|(i, c)| format!("Context {}: {}", i + 1, c))
            .collect::<Vec<_>>()
            .join("\n\n");

        let full_content = if !context_text.is_empty() {
            format!("Context:\n\n{}\n\nQuery: {}", context_text, query)
        } else {
            query.to_string()
        };

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: full_content,
        });

        Ok(messages)
    }

    pub async fn generate_response_with_messages(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String> {
        match self {
            LLMProvider::Ollama(p) => p.generate_response_with_messages(messages).await,
            LLMProvider::OpenAI(p) => p.generate_response_with_messages(messages).await,
            LLMProvider::Anthropic(p) => p.generate_response_with_messages(messages).await,
            LLMProvider::Groq(p) => p.generate_response_with_messages(messages).await,
        }
    }

    pub async fn test_connection(&self) -> Result<bool> {
        match self {
            LLMProvider::Ollama(p) => p.test_connection().await,
            LLMProvider::OpenAI(p) => p.test_connection().await,
            LLMProvider::Anthropic(p) => p.test_connection().await,
            LLMProvider::Groq(p) => p.test_connection().await,
        }
    }

    fn build_prompt(&self, query: &str, context: &[String]) -> String {
        if context.is_empty() {
            return query.to_string();
        }

        let context_text = context
            .iter()
            .enumerate()
            .map(|(i, c)| format!("Context {}: {}", i + 1, c))
            .collect::<Vec<_>>()
            .join("\n\n");

        format!(
            "Based on the following context:\n\n{}\n\nQuestion: {}\n\nAnswer:",
            context_text, query
        )
    }
}

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    url: String,
    model: String,
    temperature: f32,
}

impl OllamaProvider {
    pub fn new(url: String, model: String, temperature: f32) -> Self {
        Self {
            url: if url.ends_with('/') {
                url
            } else {
                format!("{}/", url)
            },
            model,
            temperature,
        }
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String> {
        let client = Client::new();

        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            temperature: self.temperature,
            stream: false,
        };

        let response = client
            .post(format!("{}api/generate", self.url))
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Ollama API request failed: {}\n{}",
                status,
                error_text
            ));
        }

        let body: GenerateResponse = response.json().await?;
        Ok(body.response)
    }

    pub async fn generate_response_with_messages(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String> {
        let client = Client::new();

        let messages_vec: Vec<OllamaChatMessage> = messages
            .iter()
            .map(|m| OllamaChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: messages_vec,
            stream: false,
        };

        let response = client
            .post(format!("{}api/chat", self.url))
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Ollama API request failed: {}\n{}",
                status,
                error_text
            ));
        }

        let body: OllamaChatResponse = response.json().await?;
        Ok(body.message.content)
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let client = Client::new();

        let response = client.get(format!("{}api/tags", self.url)).send().await?;

        Ok(response.status().is_success())
    }
}

#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    url: String,
    temperature: f32,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String, url: String, temperature: f32) -> Self {
        Self {
            api_key,
            model,
            url: if url.ends_with('/') {
                url
            } else {
                format!("{}/", url)
            },
            temperature,
        }
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String> {
        let client = Client::new();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.temperature,
            stream: false,
        };

        let base_url = if self.url.ends_with('/') {
            self.url.trim_end_matches('/').to_string()
        } else {
            self.url.clone()
        };

        let response = client
            .post(format!("{}/chat/completions", base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "OpenAI API request failed: {}\n{}",
                status,
                error_text
            ));
        }

        let body: ChatResponse = response.json().await?;
        Ok(body.choices[0].message.content.clone())
    }

    pub async fn generate_response_with_messages(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String> {
        let client = Client::new();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            temperature: self.temperature,
            stream: false,
        };

        let base_url = if self.url.ends_with('/') {
            self.url.trim_end_matches('/').to_string()
        } else {
            self.url.clone()
        };

        let response = client
            .post(format!("{}/chat/completions", base_url))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "OpenAI API request failed: {}\n{}",
                status,
                error_text
            ));
        }

        let body: ChatResponse = response.json().await?;
        Ok(body.choices[0].message.content.clone())
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        let base_url = if self.url.ends_with('/') {
            self.url.trim_end_matches('/').to_string()
        } else {
            self.url.clone()
        };

        let response = client
            .get(format!("{}/models", base_url))
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    temperature: f32,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String, temperature: f32) -> Self {
        Self {
            api_key,
            model,
            temperature,
        }
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String> {
        let client = Client::new();

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.temperature,
            max_tokens: 1024,
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .bearer_auth(&self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Anthropic API request failed: {}\n{}",
                status,
                error_text
            ));
        }

        let body: AnthropicResponse = response.json().await?;
        Ok(body.content[0].text.clone())
    }

    pub async fn generate_response_with_messages(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String> {
        let client = Client::new();

        let anthropic_messages: Vec<AnthropicMessage> = messages
            .iter()
            .map(|m| {
                let role = if m.role == "user" {
                    "user"
                } else {
                    "assistant"
                };
                AnthropicMessage {
                    role: role.to_string(),
                    content: m.content.clone(),
                }
            })
            .collect();

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: anthropic_messages,
            temperature: self.temperature,
            max_tokens: 1024,
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .bearer_auth(&self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Anthropic API request failed: {}\n{}",
                status,
                error_text
            ));
        }

        let body: AnthropicResponse = response.json().await?;
        Ok(body.content[0].text.clone())
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let client = Client::new();

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .bearer_auth(&self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [{"role": "user", "content": "test"}],
                "max_tokens": 1
            }))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

#[derive(Debug, Clone)]
pub struct GroqProvider {
    api_key: String,
    model: String,
    temperature: f32,
}

impl GroqProvider {
    pub fn new(api_key: String, model: String, temperature: f32) -> Self {
        Self {
            api_key,
            model,
            temperature,
        }
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String> {
        let client = Client::new();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.temperature,
            stream: false,
        };

        let response = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Groq API request failed: {}\n{}",
                status,
                error_text
            ));
        }

        let body: ChatResponse = response.json().await?;
        Ok(body.choices[0].message.content.clone())
    }

    pub async fn generate_response_with_messages(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String> {
        let client = Client::new();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            temperature: self.temperature,
            stream: false,
        };

        let response = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Groq API request failed: {}\n{}",
                status,
                error_text
            ));
        }

        let body: ChatResponse = response.json().await?;
        Ok(body.choices[0].message.content.clone())
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let client = Client::new();

        let response = client
            .get("https://api.groq.com/openai/v1/models")
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaChatMessage,
}

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Debug, Deserialize)]
struct ChatMessageContent {
    content: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}
