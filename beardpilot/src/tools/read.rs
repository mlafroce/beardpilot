use beardpilot_api::endpoint::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::tools::ToolError;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "Path to file")]
    path: String,
}

#[derive(Default)]
pub struct Read {}

impl Tool for Read {
    type Params = Params;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "Read"
    }

    fn description(&self) -> &'static str {
        "Reads file content"
    }

    async fn call(&mut self, parameters: Self::Params) -> Result<String, Self::Error> {
        let output = tokio::process::Command::new("cat")
            .args([&parameters.path])
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            Ok(format!("cat failed: {}", stderr))
        }
    }
}
