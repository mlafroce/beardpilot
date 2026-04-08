use crossterm::event::EventStream;
use futures_util::StreamExt;
use ollama_rs::Ollama;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::task::JoinSet;
use tracing::debug;

use crate::chat::conversation::{Message, ModelInfo};
use crate::chat::session::Session;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::event::{AppEvent, SessionEvent, UiAction};
use crate::ui::tui::Tui;

#[derive(Default)]
pub struct AppState {
    pub messages: Vec<Message>,
    pub model_info: ModelInfo,
    //running_status
}

/// Top-level application struct that owns all components and drives the main loop.
pub struct App {
    tui: Tui,
    config: AppConfig,
    state: AppState,
}

impl App {
    pub fn new(config: AppConfig) -> Result<Self, AppError> {
        let tui = Tui::new().map_err(AppError::Io)?;
        let state = Default::default();
        Ok(Self {
            config,
            tui,
            state,
        })
    }

    /// Run the interactive chat loop until the user exits.
    pub async fn run(&mut self) -> Result<(), AppError> {
        // Initial render
        let ollama = Ollama::new(self.config.host.to_string(), self.config.port);
        let models = ollama.list_local_models().await?;
        debug!("Local models:");
        debug!("{:?}", models);
        let (sender, mut receiver) = unbounded_channel();
        let mut tasks = tokio::task::JoinSet::new();
        let session_sender = App::spawn_session_actor(&mut tasks, self.config.clone(), ollama);

        let sender_clone = sender.clone();
        tokio::spawn(async move {
            let mut events = EventStream::new();
            while let Some(event) = events.next().await {
                if sender_clone.send(AppEvent::UiEvent(event.unwrap())).is_err() { break; }
            }
        });

        self.redraw()?;
        loop {
            let event = receiver.recv().await;
            match event {
                Some(AppEvent::UiEvent(ui_event)) => {
                    let action = self.tui.handle_event(ui_event);
                    match action {
                        UiAction::Quit => break,
                        UiAction::Submit(text) => {
                            debug!("Submitted: {text}");
                            let _ = session_sender.send(SessionEvent::SubmitPrompt(text, sender.clone()));
                        }
                        UiAction::Redraw => {
                            self.redraw()?;
                        }
                        _ => {}
                    }
                },
                Some(AppEvent::MessageAdded(msg)) => {
                    self.state.messages.push(msg);
                    self.redraw()?;
                }
                Some(AppEvent::SubmitResponse(response)) => {
                    debug!("Response arrived: {:?}", response);
                    if let Err(e) = response {
                        debug!("Submit error: {:?}", e);
                    }
                }
                _ => {}
            }
        };
        Ok(())
    }

    fn spawn_session_actor(tasks: &mut JoinSet<()>, config: AppConfig, ollama: Ollama) -> UnboundedSender<SessionEvent> {
        let actor = Session::new(&config, ollama).unwrap();
        let sender = actor.get_sender();
        tasks.spawn(actor.run());
        sender
    }

    fn redraw(&mut self) -> Result<(), AppError> {
        self.tui
            .draw(&self.state)
            .map_err(AppError::Io)?;
        Ok(())
    }
}
