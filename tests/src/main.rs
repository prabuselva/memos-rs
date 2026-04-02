use clap::Parser;
use memos_rs::{
    create_app_router_with_model, AppState, BERTModel, Config, ModelDownloader, VERSION,
    VERSION_SHORT,
};
use std::sync::Arc;

use tower_http::cors::CorsLayer;
use tracing::{error, info, Level};

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = load_config(cli.config.as_deref())?;

    init_logging(&config.logging);

    let downloader = ModelDownloader::new(&config.vector.model_cache_dir);
    downloader.ensure_model_downloaded()?;

    let tokenizer_path = config.get_tokenizer_path();
    let model_dir = tokenizer_path.parent().unwrap();

    let model = BERTModel::from_tokenizer(
        &tokenizer_path.to_string_lossy(),
        &model_dir.to_string_lossy(),
    )
    .unwrap_or_else(|_| {
        error!("Failed to load embedding model, continuing without LLM features");
        std::process::exit(1);
    });

    tokio::runtime::Runtime::new()?.block_on(async {
        let model_arc = Arc::new(model);
        let state = Arc::new(AppState::new(config, model_arc.clone()).await?);
        let app = create_app_router_with_model(model_arc)
            .with_state(state)
            .layer(CorsLayer::permissive());

        let addr = format!("0.0.0.0:{}", cli.port);
        info!("Starting server on http://{}", addr);
        info!("Version: {} (short: {})", VERSION, VERSION_SHORT);

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
