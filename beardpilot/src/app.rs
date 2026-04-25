use beardpilot_api::client::mistral::Mistral;
use crossterm::event::EventStream;
use futures_util::StreamExt;
use tokio::sync::mpsc::{self, unbounded_channel, UnboundedSender};
use tokio::task::JoinSet;
use tracing::debug;

use crate::chat::conversation::{Conversation, ModelInfo};
use crate::chat::session::Session;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::event::{AppEvent, SessionEvent, UiAction};
use crate::ui::tui::Tui;

pub struct AppState {
    pub conversation: Conversation,
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
        let model_info = ModelInfo {
            model_name: config.model.clone(),
            max_tokens: None,
        };
        let conversation = Conversation::new(config.system_prompt.clone(), model_info);
        let state = AppState { conversation };
        Ok(Self { config, tui, state })
    }

    /// Run the interactive chat loop until the user exits.
    pub async fn run(&mut self) -> Result<(), AppError> {
        // Initial render
        let mistral = Mistral::new(&self.config.host, self.config.api_key.as_ref().unwrap())?;
        let (sender, mut receiver) = unbounded_channel();
        let mut tasks = tokio::task::JoinSet::new();
        let session_sender = App::spawn_session_actor(&mut tasks, mistral, sender.clone());

        tokio::spawn(async move {
            let mut events = EventStream::new();
            while let Some(event) = events.next().await {
                if sender.send(AppEvent::UiEvent(event.unwrap())).is_err() {
                    break;
                }
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
                            self.state.conversation.push_user(text);
                            let _ = session_sender.send(SessionEvent::SendChat(
                                self.state.conversation.session_chat(),
                            ));
                        }
                        _ => {}
                    }
                    self.redraw()?;
                }
                Some(AppEvent::ResponseChunk(chunk)) => {
                    debug!("Received chunk: {:?}", chunk);
                    self.state.conversation.push_chunk(chunk);
                    self.tui.scroll_to_bottom();
                    self.redraw()?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn spawn_session_actor(
        tasks: &mut JoinSet<()>,
        ollama: Mistral,
        app_sender: mpsc::UnboundedSender<AppEvent>,
    ) -> UnboundedSender<SessionEvent> {
        let actor = Session::new(ollama, app_sender).unwrap();
        let sender = actor.get_sender();
        tasks.spawn(actor.run());
        sender
    }

    fn redraw(&mut self) -> Result<(), AppError> {
        self.tui.draw(&self.state).map_err(AppError::Io)?;
        Ok(())
    }
}
