use beardpilot_api::endpoint::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::tools::ToolError;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "Command to execute")]
    command: String,
}

#[derive(Default)]
pub struct Bash {}

impl Tool for Bash {
    type Params = Params;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "Bash"
    }

    fn description(&self) -> &'static str {
        "Executes commands via bash"
    }

    async fn call(&mut self, parameters: Self::Params) -> Result<String, Self::Error> {
        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&parameters.command)
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            Ok(format!("Bash execution failed: {}", stderr))
        }
    }
}
