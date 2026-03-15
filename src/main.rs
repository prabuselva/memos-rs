use clap::Parser;
use memos_rs::{AppState, Config, create_app_router, VERSION, VERSION_SHORT};
use std::sync::Arc;

use tower_http::cors::CorsLayer;

#[derive(Parser)]
#[command(name = "memos-rs")]
#[command(about = "A Joplin-like note-taking application")]
#[command(version = VERSION)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = load_config(cli.config.as_deref())?;
    let state = Arc::new(AppState::new(config).await?);
    let app = create_app_router().with_state(state).layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", cli.port);
    println!("Starting server on http://{}", addr);
    println!("Version: {} (short: {})", VERSION, VERSION_SHORT);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn load_config(path: Option<&str>) -> anyhow::Result<Config> {
    let config_path = path.unwrap_or("config.toml");

    if std::path::Path::new(config_path).exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;
        return Ok(config);
    }

    Ok(Config::default())
}