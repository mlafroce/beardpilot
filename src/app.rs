use crossterm::event::EventStream;
use futures_util::StreamExt;
use ollama_rs::Ollama;
use tracing::debug;

use crate::chat::session::Session;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::event::AppAction;
use crate::ui::tui::Tui;

/// Top-level application struct that owns all components and drives the main loop.
pub struct App {
    tui: Tui,
    config: AppConfig,
    session: Option <Session>
}

impl App {
    pub fn new(config: AppConfig) -> Result<Self, AppError> {
        let tui = Tui::new().map_err(AppError::Io)?;
        Ok(Self {
            config,
            tui,
            session: None,
        })
    }

    /// Run the interactive chat loop until the user exits.
    pub async fn run(&mut self) -> Result<(), AppError> {
        // Initial render
        let ollama = Ollama::new(self.config.host.to_string(), self.config.port);
        let models = ollama.list_local_models().await?;
        debug!("Local models:");
        debug!("{:?}", models);
        self.session = Some(Session::new(&self.config, ollama).await?);
        self.redraw()?;

        let mut events = EventStream::new();

        while let Some(event) = events.next().await {
            let action = self.tui.handle_event(event?);
            match action {
                AppAction::Quit => break,
                AppAction::Submit(text) => {
                    self.submit(text).await?;
                    self.redraw()?;
                }
                AppAction::Redraw => {
                    self.redraw()?;
                }
                AppAction::None => {}
            }
        }

        Ok(())
    }

    fn redraw(&mut self) -> Result<(), AppError> {
        if let Some(session) = &mut self.session {
            self.tui
                .draw(&session.messages, false, session.conversation.model_info())
                .map_err(AppError::Io)?;
        }
        Ok(())
    }

    async fn submit(&mut self, text: String) -> Result<(), AppError> {
        if let Some(session) = &mut self.session {
            session.submit(text).await?;
        }
        Ok(())
    }
}
