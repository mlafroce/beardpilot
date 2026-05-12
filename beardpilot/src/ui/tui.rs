use crossterm::{
    cursor,
    event::{
        DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};

use crate::{
    app::AppState,
    chat::conversation::{ModelInfo, ResponseStatus},
    event::UiAction,
    ui::{input::TextInput, message_area::TuiMainArea},
};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    message_area: TuiMainArea,
    input: TextInput,
}

impl Tui {
    /// Initialise the terminal: raw mode, alternate screen, mouse capture.
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen,)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            input: TextInput::new(),
            message_area: TuiMainArea::default(),
        })
    }

    /// Restore the terminal to its original state.
    pub fn restore(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Render the full TUI frame.
    ///
    /// * `messages`   – chat history to display.
    /// * `thinking`   – when `true` a "thinking…" indicator replaces the cursor.
    /// * `model_info` – model metadata shown below the input box.
    pub fn draw(&mut self, state: &AppState) -> io::Result<()> {
        // Build model status label outside the closure to avoid borrow issues.
        let status_label = build_model_status(&state.conversation.model_info);

        let pending_calls = state.tool_registry.peek_pending_calls();
        let notification = pending_calls.map(|tc| tc.to_string());

        self.terminal.draw(|frame| {
            let (msgs_area, notif_area, input_area) =
                split_layout(frame.area(), notification.is_some());
            if let Some(content) = &state.main_area_text {
                self.message_area.render_text(frame, msgs_area, content);
            } else {
                self.message_area
                    .render_conversation(frame, msgs_area, &state.conversation);
            }
            if let (Some(area), Some(text)) = (notif_area, &notification) {
                render_notification(frame, area, text);
            }

            let res_status = state.conversation.conversation_status();
            TuiInput::render(frame, input_area, &self.input, res_status, &status_label);
        })?;

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event) -> UiAction {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            _ => UiAction::None,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> UiAction {
        if matches!(
            key,
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } | KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }
        ) {
            return UiAction::Quit;
        }

        match key.code {
            KeyCode::Enter => {
                let text = self.input.take();
                let text = text.trim().to_string();

                if text.is_empty() {
                    return UiAction::None;
                }
                return UiAction::Submit(text);
            }
            // editing
            KeyCode::Char(c) => {
                self.input.insert(c);
            }
            KeyCode::Backspace => {
                self.input.delete_before();
            }
            KeyCode::Delete => {
                self.input.delete_after();
            }
            // Cursor
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.input.move_start();
                } else {
                    self.input.move_left();
                }
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.input.move_end();
                } else {
                    self.input.move_right();
                }
            }
            KeyCode::Home => {
                self.input.move_start();
            }
            KeyCode::End => {
                self.input.move_end();
            }
            KeyCode::Up | KeyCode::PageUp => {
                self.message_area
                    .scroll_up(if key.code == KeyCode::PageUp { 10 } else { 3 });
            }
            KeyCode::Down | KeyCode::PageDown => {
                self.message_area
                    .scroll_down(if key.code == KeyCode::PageDown { 10 } else { 3 });
            }

            _ => {}
        }
        UiAction::None
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> UiAction {
        let input_area = self.input_area();
        let msgs_area = self.messages_area();

        match mouse.kind {
            // Click inside the input box → reposition cursor
            MouseEventKind::Down(MouseButton::Left)
                if mouse.row >= input_area.top()
                    && mouse.row < input_area.bottom()
                    && mouse.column >= input_area.left()
                    && mouse.column < input_area.right() =>
            {
                // column relative to the inner widget (subtract left border)
                let col = mouse.column.saturating_sub(input_area.x + 1);
                self.input.set_cursor_from_click(col);
            }

            // Scroll wheel in the messages pane
            MouseEventKind::ScrollUp if mouse.row < msgs_area.bottom() => {
                self.message_area.scroll_up(3);
            }
            MouseEventKind::ScrollDown if mouse.row < msgs_area.bottom() => {
                self.message_area.scroll_down(3);
            }

            _ => {}
        }
        UiAction::None
    }

    /// Returns the terminal area of the messages pane so callers can check
    /// whether a mouse event landed inside it.
    pub fn messages_area(&self) -> Rect {
        let size = self.terminal.size().unwrap_or_default();
        let area = Rect::new(0, 0, size.width, size.height);
        split_layout(area, false).0
    }

    /// Returns the terminal area of the input pane.
    pub fn input_area(&self) -> Rect {
        let size = self.terminal.size().unwrap_or_default();
        let area = Rect::new(0, 0, size.width, size.height);
        split_layout(area, false).2
    }

    pub fn scroll_to_bottom(&mut self) {
        self.message_area.scroll_to_bottom();
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

// ── layout helpers ─────────────────────────────────────────────────────────────

/// Split the terminal into [messages_area, optional_notification_area, input_area].
/// The input area is 4 rows tall: 3 for the bordered box + 1 for the status line.
/// When `has_notification` is true an extra 3-row bordered notification strip is
/// inserted between the messages pane and the input area.
fn split_layout(area: Rect, has_notification: bool) -> (Rect, Option<Rect>, Rect) {
    if has_notification {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(4),
            ])
            .split(area);
        (chunks[0], Some(chunks[1]), chunks[2])
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(4)])
            .split(area);
        (chunks[0], None, chunks[1])
    }
}

// ── rendering helpers ──────────────────────────────────────────────────────────

/// Build a one-line status string from `ModelInfo`.
fn build_model_status(info: &ModelInfo) -> String {
    match info.max_tokens {
        Some(max) => format!(" model: {}  │  max tokens: {} ", info.model_name, max),
        None => format!(" model: {} ", info.model_name),
    }
}

/// Render a notification strip between the messages pane and the input box.
fn render_notification(frame: &mut Frame, area: Rect, text: &str) {
    let paragraph = Paragraph::new(Span::styled(
        format!(" ⚠  {} ", text),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" notification "),
    );
    frame.render_widget(paragraph, area);
}

pub struct TuiInput;

impl TuiInput {
    /// Render the input box and a status line below it.
    ///
    /// `area` must be at least 4 rows tall: 3 for the bordered input box + 1 for
    /// the model-info status line.
    fn render(
        frame: &mut Frame,
        area: Rect,
        input: &TextInput,
        res_status: ResponseStatus,
        status_label: &str,
    ) {
        // Split the area: top 3 rows → input box, bottom 1 row → status line.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(1)])
            .split(area);
        let box_area = chunks[0];
        let status_area = chunks[1];

        // ── input box ──────────────────────────────────────────────────────────
        let title = match res_status {
            ResponseStatus::ReceiveResponse => " reading ",
            ResponseStatus::Thinking => " thinking ",
            ResponseStatus::Waiting => " message ",
        };
        let block = Block::default().borders(Borders::ALL).title(title);
        let inner = block.inner(box_area);

        let prompt = "> ";
        let prompt_len = prompt.len() as u16;
        let display_text = format!("{}{}", prompt, input.as_str());

        let paragraph = Paragraph::new(display_text).block(block);
        frame.render_widget(paragraph, box_area);

        // Position the real terminal cursor (only when not in thinking mode)
        if res_status == ResponseStatus::Waiting {
            let cursor_x = inner.x + prompt_len + input.cursor() as u16;
            let cursor_x = cursor_x.min(inner.x + inner.width.saturating_sub(1));
            frame.set_cursor_position((cursor_x, inner.y));
        }

        // ── status line ────────────────────────────────────────────────────────
        let status = Paragraph::new(Span::styled(
            status_label.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(status, status_area);
    }
}
