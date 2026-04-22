mod app;
mod chat;
mod config;
mod error;
mod event;
mod tools;
mod ui;

use app::App;
use config::AppConfig;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    let file_appender = tracing_appender::rolling::daily("logs", "tui.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(env_filter)
        .init();
    App::new(config)?.run().await?;
    Ok(())
}
