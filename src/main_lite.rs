use clap::Parser;
use memos_rs::{AppStateLite, Config, VERSION, VERSION_SHORT};
use std::sync::Arc;

use tower_http::cors::CorsLayer;
use tracing::{info, Level};

#[derive(Parser)]
#[command(name = "memos-rs-lite")]
#[command(about = "A lightweight Joplin-like note-taking application (SQLite only)")]
#[command(version = VERSION)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = load_config(cli.config.as_deref())?;

    init_logging(&config.logging);

    tokio::runtime::Runtime::new()?.block_on(async {
        let state = Arc::new(AppStateLite::new(config).await?);
        let app = memos_rs::create_app_router_lite()
            .with_state(state)
            .layer(CorsLayer::permissive());

        let addr = format!("0.0.0.0:{}", cli.port);
        info!("Starting server on http://{}", addr);
        info!("Version: {} (short: {})", VERSION, VERSION_SHORT);
        info!("Running in LITE mode (SQLite only, no vector search)");

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(()) as anyhow::Result<()>
    })?;

    Ok(())
}

fn init_logging(logging_config: &memos_rs::config::LoggingConfig) {
    let log_level = match logging_config.level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .compact()
        .with_max_level(log_level)
        .init();
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
