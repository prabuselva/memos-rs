use anyhow::Result;
use memos_rs::llm::{generate_llm_response, test_llm_connection};
use memos_rs::models::auth_dto::LLMSettings;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing OpenAI LLM Provider");
    println!("===========================\n");

    let settings = LLMSettings {
        provider: "openai".to_string(),
        url: "http://192.168.0.87:8083".to_string(),
        api_key: Some("123456".to_string()),
        model: "gpt-3.5-turbo".to_string(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    println!("Settings:");
    println!("  Provider: {}", settings.provider);
    println!("  URL: {}", settings.url);
    println!("  Model: {}", settings.model);
    println!("  Temperature: {}", settings.temperature);
    println!("  Max Tokens: {}", settings.max_tokens);
    println!();

    println!("Testing connection...");
    match test_llm_connection(&settings).await {
        Ok(success) => {
            if success {
                println!("✅ Connection test successful!");
            } else {
                println!("❌ Connection test failed!");
            }
        }
        Err(e) => {
            println!("❌ Connection test error: {}", e);
        }
    }

    println!();
    println!("Testing response generation...");
    let query = "Hello, how are you?";
    let context: Vec<String> = vec![];

    match generate_llm_response(&settings, query, &context).await {
        Ok(response) => {
            println!("✅ Response generated:");
            println!("   Query: {}", query);
            println!("   Response: {}", response);
        }
        Err(e) => {
            println!("❌ Response generation error: {}", e);
        }
    }

    Ok(())
}
