pub enum CommandOutput {
    Finished,
    Info(String),
}

/// Slash commands used in chat
pub trait SlashCommand {
    /// Command name to be used
    fn get_name(&self) -> String;
    /// Prompt to be show when activated
    fn start(&self, arg: String) -> String;
    /// Temporal method: process input
    fn handle_input(&mut self, input: String) -> CommandOutput;
    /// Get the text content to display for this command
    fn get_text(&self) -> String;
}
