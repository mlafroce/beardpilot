use crate::{
    app::AppState,
    chat::conversation::{Conversation, LocalMessage},
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

#[derive(Default)]
pub struct TuiMainArea {
    /// Scroll offset for the messages pane (in lines).
    scroll: u16,
    /// Cached height of the last-rendered messages area (in terminal rows).
    messages_area_height: u16,
    /// Cached total number of rendered lines in the messages pane.
    total_messages_lines: u16,
}

impl TuiMainArea {
    pub fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        // We capture the two cached values from the draw call.
        let messages_area_height = area.height;

        let lines = Self::build_message_lines(&state.conversation, area.width.saturating_sub(2));
        let total_messages_lines = lines.len() as u16;

        let msgs_paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" beardpilot ")
                    .title_style(Style::default().add_modifier(Modifier::BOLD)),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));
        frame.render_widget(msgs_paragraph, area);
        self.messages_area_height = messages_area_height;
        self.total_messages_lines = total_messages_lines;
    }
    /// Build all `Line`s to display in the messages pane.
    ///
    /// Long messages are pre-wrapped at `max_width` columns so the scroll-line
    /// count stays accurate.
    fn build_message_lines(conversation: &Conversation, max_width: u16) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        for msg in conversation.messages() {
            let mut msg_lines = Self::build_message_line(msg, max_width);
            lines.append(&mut msg_lines);
            // blank line between messages
            lines.push(Line::from(""));
        }

        lines
    }

    fn build_message_line(msg: &LocalMessage, max_width: u16) -> Vec<Line<'static>> {
        let (prefix, prefix_style, text_style, text) = match &msg {
            LocalMessage::User(content) => (
                "You  │ ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(Color::White),
                content,
            ),
            LocalMessage::Assistant(content) => (
                "AI   │ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(Color::White),
                content,
            ),
            LocalMessage::Info(content) => (
                "Info │ ",
                Style::default().fg(Color::Yellow),
                Style::default().fg(Color::DarkGray),
                content,
            ),
            /*LocalMessage::Error => (
                "Err  │ ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Red),

            ),*/
            LocalMessage::Thinking(content) => (
                "Think│ ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(Color::DarkGray),
                content,
            ),
            LocalMessage::ToolCall(tc) => (
                "Tool │ ",
                Style::default().fg(Color::Yellow),
                Style::default().fg(Color::DarkGray),
                &tc.to_string(),
            ),
            LocalMessage::ToolResponse { id: _, response } => (
                "Tool │ ",
                Style::default().fg(Color::Yellow),
                Style::default().fg(Color::DarkGray),
                response,
            ),
        };

        // The content area after the prefix
        let prefix_len = prefix.chars().count() as u16;
        let text_width = max_width.saturating_sub(prefix_len).max(1) as usize;

        let wrapped = Self::soft_wrap(text, text_width);

        let mut lines = vec![];
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

    // ── scroll helpers ─────────────────────────────────────────────────────

    pub fn scroll_up(&mut self, lines: u16) {
        self.scroll = self.scroll.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: u16) {
        let total = self.total_messages_lines();
        let visible = self.messages_area_height();
        let max_scroll = total.saturating_sub(visible);
        self.scroll = (self.scroll + lines).min(max_scroll);
    }

    /// Scroll the messages pane to the very bottom.
    pub fn scroll_to_bottom(&mut self) {
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
}
