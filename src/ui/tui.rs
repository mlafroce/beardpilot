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
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use tracing::debug;

use crate::{
    app::AppState,
    chat::conversation::{Conversation, ModelInfo, ResponseStatus, Role},
    event::UiAction,
    ui::input::TextInput,
};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    input: TextInput,
    /// Cached height of the last-rendered messages area (in terminal rows).
    messages_area_height: u16,
    /// Cached total number of rendered lines in the messages pane.
    total_messages_lines: u16,
    /// Scroll offset for the messages pane (in lines).
    scroll: u16,
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
            messages_area_height: 0,
            total_messages_lines: 0,
            scroll: 0,
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
        // We capture the two cached values from the draw call.
        let mut messages_area_height = self.messages_area_height;
        let mut total_messages_lines = self.total_messages_lines;

        // Build model status label outside the closure to avoid borrow issues.
        let status_label = build_model_status(&state.model_info);

        self.terminal.draw(|frame| {
            let (msgs_area, input_area) = split_layout(frame.area());
            messages_area_height = msgs_area.height;

            let lines = build_message_lines(&state.conversation, msgs_area.width.saturating_sub(2));
            total_messages_lines = lines.len() as u16;

            let msgs_paragraph = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" beardpilot ")
                        .title_style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .wrap(Wrap { trim: false })
                .scroll((self.scroll, 0));
            frame.render_widget(msgs_paragraph, msgs_area);

            let res_status = state.conversation.response_status();
            render_input(frame, input_area, &self.input, res_status, &status_label);
        })?;

        self.messages_area_height = messages_area_height;
        self.total_messages_lines = total_messages_lines;

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
        // Global quit shortcuts always work, even while thinking
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

        //if self.thinking {
        //    return Ok(false);
        //}

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
                self.scroll_up(if key.code == KeyCode::PageUp { 10 } else { 3 });
            }
            KeyCode::Down | KeyCode::PageDown => {
                self.scroll_down(if key.code == KeyCode::PageDown { 10 } else { 3 });
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
            MouseEventKind::Down(MouseButton::Left) => {
                if mouse.row >= input_area.top()
                    && mouse.row < input_area.bottom()
                    && mouse.column >= input_area.left()
                    && mouse.column < input_area.right()
                {
                    // column relative to the inner widget (subtract left border)
                    let col = mouse.column.saturating_sub(input_area.x + 1);
                    self.input.set_cursor_from_click(col);
                }
            }

            // Scroll wheel in the messages pane
            MouseEventKind::ScrollUp => {
                if mouse.row < msgs_area.bottom() {
                    self.scroll_up(3);
                }
            }
            MouseEventKind::ScrollDown => {
                if mouse.row < msgs_area.bottom() {
                    self.scroll_down(3);
                }
            }

            _ => {}
        }
        UiAction::None
    }

    // ── scroll helpers ─────────────────────────────────────────────────────

    fn scroll_up(&mut self, lines: u16) {
        self.scroll = self.scroll.saturating_sub(lines);
    }

    fn scroll_down(&mut self, lines: u16) {
        let total = self.total_messages_lines();
        let visible = self.messages_area_height();
        let max_scroll = total.saturating_sub(visible);
        self.scroll = (self.scroll + lines).min(max_scroll);
    }

    /// Scroll the messages pane to the very bottom.
    fn scroll_to_bottom(&mut self) {
        let total = self.total_messages_lines();
        let visible = self.messages_area_height();
        self.scroll = total.saturating_sub(visible);
    }

    /// Height of the messages area from the last draw call (inner, excluding borders).
    pub fn messages_area_height(&self) -> u16 {
        self.messages_area_height.saturating_sub(2) // subtract top+bottom border
    }

    /// Total lines rendered in the messages pane from the last draw call.
    pub fn total_messages_lines(&self) -> u16 {
        self.total_messages_lines
    }

    /// Returns the terminal area of the messages pane so callers can check
    /// whether a mouse event landed inside it.
    pub fn messages_area(&self) -> Rect {
        let size = self.terminal.size().unwrap_or_default();
        let area = Rect::new(0, 0, size.width, size.height);
        split_layout(area).0
    }

    /// Returns the terminal area of the input pane.
    pub fn input_area(&self) -> Rect {
        let size = self.terminal.size().unwrap_or_default();
        let area = Rect::new(0, 0, size.width, size.height);
        split_layout(area).1
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

// ── layout helpers ─────────────────────────────────────────────────────────────

/// Split the terminal into [messages_area, input_area].
/// The input area is 4 rows tall: 3 for the bordered box + 1 for the status line.
fn split_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(4)])
        .split(area);
    (chunks[0], chunks[1])
}

// ── rendering helpers ──────────────────────────────────────────────────────────

/// Build all `Line`s to display in the messages pane.
///
/// Long messages are pre-wrapped at `max_width` columns so the scroll-line
/// count stays accurate.
fn build_message_lines(conversation: &Conversation, max_width: u16) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for msg in conversation.messages() {
        let (prefix, prefix_style, text_style) = match msg.role {
            Role::User => (
                "You  │ ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(Color::White),
            ),
            Role::Assistant => (
                "AI   │ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(Color::White),
            ),
            Role::Info => (
                "Info │ ",
                Style::default().fg(Color::Yellow),
                Style::default().fg(Color::DarkGray),
            ),
            Role::Error => (
                "Err  │ ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Red),
            ),
            Role::Thinking => (
                "Think│ ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(Color::DarkGray),
            ),
        };

        // The content area after the prefix
        let prefix_len = prefix.chars().count() as u16;
        let text_width = max_width.saturating_sub(prefix_len).max(1) as usize;

        let text = msg.text.clone();
        let wrapped = soft_wrap(&text, text_width);

        for (i, segment) in wrapped.iter().enumerate() {
            if i == 0 {
                lines.push(Line::from(vec![
                    Span::styled(prefix.to_string(), prefix_style),
                    Span::styled(segment.clone(), text_style),
                ]));
            } else {
                // continuation lines – indent by prefix width
                let indent = " ".repeat(prefix_len as usize);
                lines.push(Line::from(vec![
                    Span::styled(indent, Style::default()),
                    Span::styled(segment.clone(), text_style),
                ]));
            }
        }
        // blank line between messages
        lines.push(Line::from(""));
    }

    lines
}

/// Break `text` into lines of at most `width` columns, respecting existing
/// newlines.
fn soft_wrap(text: &str, width: usize) -> Vec<String> {
    let mut result = Vec::new();
    for raw_line in text.split('\n') {
        if raw_line.is_empty() {
            result.push(String::new());
            continue;
        }
        let chars: Vec<char> = raw_line.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let end = (start + width).min(chars.len());
            result.push(chars[start..end].iter().collect());
            start = end;
        }
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

/// Build a one-line status string from `ModelInfo`.
fn build_model_status(info: &ModelInfo) -> String {
    match info.max_tokens {
        Some(max) => format!(" model: {}  │  max tokens: {} ", info.model_name, max),
        None => format!(" model: {} ", info.model_name),
    }
}

/// Render the input box and a status line below it.
///
/// `area` must be at least 4 rows tall: 3 for the bordered input box + 1 for
/// the model-info status line.
fn render_input(
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
        ResponseStatus::Waiting => " message "
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
