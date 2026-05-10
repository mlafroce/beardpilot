use beardpilot_api::client::mistral::MistralClient;
use crossterm::event::EventStream;
use futures_util::StreamExt;
use tokio::sync::mpsc::{self, unbounded_channel, UnboundedSender};
use tokio::task::JoinSet;

use crate::chat::conversation::{Conversation, ModelInfo, ResponseAction};
use crate::chat::session::Session;
use crate::chat::tool_registry::ToolRegistry;
use crate::config::AppConfig;
use crate::error::{AppError, AppResult};
use crate::event::{AppEvent, SessionEvent, UiAction};
use crate::ui::tui::Tui;

pub struct AppState {
    pub conversation: Conversation,
    pub tool_registry: ToolRegistry,
}

/// Top-level application struct that owns all components and drives the main loop.
pub struct App {
    tui: Tui,
    config: AppConfig,
    state: AppState,
}

impl App {
    pub fn new(config: AppConfig) -> AppResult<Self> {
        let tui = Tui::new().map_err(AppError::Io)?;
        let model_info = ModelInfo {
            model_name: config.model.clone(),
            max_tokens: None,
        };
        let conversation = Conversation::new(config.system_prompt.clone(), model_info);
        let tool_registry = ToolRegistry::new();
        let state = AppState {
            conversation,
            tool_registry,
        };
        Ok(Self { config, tui, state })
    }

    /// Run the interactive chat loop until the user exits.
    pub async fn run(&mut self) -> AppResult<()> {
        // Initial render
        let mistral = MistralClient::new(&self.config.host, self.config.api_key.as_ref().unwrap())?;
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
                            self.handle_submit(&session_sender, text).await;
                        }
                        _ => {}
                    }
                    self.redraw()?;
                }
                Some(AppEvent::ResponseChunk(chunk)) => {
                    let action = self.state.conversation.push_chunk(chunk.clone())?;
                    if let ResponseAction::ToolCalls(tool_calls) = action {
                        self.state.tool_registry.queue_tool_calls(tool_calls);
                    }
                    self.tui.scroll_to_bottom();
                    self.redraw()?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_submit(
        &mut self,
        session_sender: &UnboundedSender<SessionEvent>,
        text: String,
    ) {
        match self.state.tool_registry.pop_pending_call() {
            Some(tool_call) => {
                if text == "y" {
                    let id = tool_call.id.clone();
                    self.state.conversation.push_tool_call(tool_call.clone());
                    let response = self.state.tool_registry.call_tool(tool_call).await.unwrap();
                    self.state.conversation.push_tool_response(id, response);
                }
            }
            None => {
                self.state.conversation.push_user(text);
            }
        }
        let _ = session_sender.send(SessionEvent::SendChat(
            self.state
                .conversation
                .session_chat(&self.state.tool_registry),
        ));
    }

    fn spawn_session_actor(
        tasks: &mut JoinSet<()>,
        client: MistralClient,
        app_sender: mpsc::UnboundedSender<AppEvent>,
    ) -> UnboundedSender<SessionEvent> {
        let actor = Session::new(client, app_sender).unwrap();
        let sender = actor.get_sender();
        tasks.spawn(actor.run());
        sender
    }

    fn redraw(&mut self) -> Result<(), AppError> {
        self.tui.draw(&self.state).map_err(AppError::Io)?;
        Ok(())
    }
}
