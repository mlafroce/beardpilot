use beardpilot_api::endpoint::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::tools::ToolError;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "Expression to find in folder or file. Accepts regex.")]
    expression: String,
    path: Option<String>,
}

pub struct Find {}

impl Tool for Find {
    type Params = Params;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "Find"
    }

    fn description(&self) -> &'static str {
        "Finds an expression in a file or folder"
    }

    async fn call(&mut self, parameters: Self::Params) -> Result<String, Self::Error> {
        let search_path = parameters.path.as_deref().unwrap_or(".");
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
            Ok(format!("grep failed: {}", stderr))
        }
    }
}
