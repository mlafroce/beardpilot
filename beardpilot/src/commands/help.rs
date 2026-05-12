use crate::chat::command::{CommandOutput, SlashCommand};

#[derive(Clone)]
pub struct HelpCommand;

impl SlashCommand for HelpCommand {
    fn get_name(&self) -> String {
        "help".to_owned()
    }

    fn start(&self, _arg: String) -> String {
        "This is Beardpilot help :)".to_owned()
    }

    fn handle_input(&mut self, _input: String) -> CommandOutput {
        CommandOutput::Finished
    }

    fn get_text(&self) -> String {
        "This is Beardpilot help :)".to_owned()
    }
}
