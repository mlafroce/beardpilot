mod app;
mod chat;
mod config;
mod error;
mod event;
mod tools;
mod ui;

use app::App;
use config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    App::new(config)?.run().await?;
    Ok(())
}
