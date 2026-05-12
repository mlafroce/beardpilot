use crate::chat::command::{CommandOutput, SlashCommand};
use crate::commands::help::HelpCommand;

/// Registry for managing slash commands in the chat application.
pub struct CommandRegistry {
    active_command: Option<Box<dyn SlashCommand>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            active_command: None,
        }
    }

    /// Activate a command by name with the given arguments.
    ///
    /// # Arguments
    /// * `command_name` - The name of the command to activate
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// The output string from the command's start method, or an error message if the command doesn't exist.
    pub fn activate(&mut self, command_name: &str, arg: &str) -> CommandOutput {
        match command_name {
            "help" => {
                let command = HelpCommand {};
                let res = command.start(arg.to_owned());
                self.active_command = Some(Box::new(command));
                CommandOutput::Info(res)
            }
            _ => CommandOutput::Info(format!("Unknown command: {}", command_name)),
        }
    }

    /// Get a reference to the currently active command, if any.
    ///
    /// # Returns
    /// An optional reference to the active SlashCommand.
    pub fn handle_input(&mut self, input: String) -> Option<CommandOutput> {
        let output = self
            .active_command
            .as_mut()
            .map(|command| command.handle_input(input));
        if let Some(CommandOutput::Finished) = output {
            self.active_command = None;
        }
        output
    }
}
