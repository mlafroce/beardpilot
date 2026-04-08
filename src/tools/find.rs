use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;
use tracing::debug;

use crate::event::SessionEvent;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "Expression to find in folder or file. Accepts regex.")]
    expression: String,
    #[schemars(
        description = "Path to the file or folder to search in. Defaults to current directory if not provided."
    )]
    path: Option<String>,
}

pub struct Find {
    sender: UnboundedSender<SessionEvent>
}

impl Find {
    pub fn new(sender: UnboundedSender<SessionEvent>) -> Self {
        Self {sender}
    }
}

impl Tool for Find {
    type Params = Params;

    fn name() -> &'static str {
        "Find"
    }

    fn description() -> &'static str {
        "Finds an expression in a file or folder"
    }

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let search_path = parameters.path.as_deref().unwrap_or(".");
        //let (conf_sender, response) = tokio::sync::oneshot::channel();
        //let prompt = format!("Confirm 'grep -rn {}'", parameters.expression);
        //self.sender.send(SessionEvent::ConfirmationRequest{prompt, response: conf_sender}).unwrap();

        let output = tokio::process::Command::new("grep")
            .args(["-rn", &parameters.expression, search_path])
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else if output.status.code() == Some(1) {
            // grep exits with 1 when no matches are found
            Ok(format!("No matches found for '{}'", parameters.expression))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            Ok(format!("grep failed: {}", stderr).into())
        }
    }
}
