use beardpilot_api::endpoint::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::tools::ToolError;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    path: Option<String>,
    show_hidden: bool,
}

#[derive(Default)]
pub struct ListFiles;

impl Tool for ListFiles {
    type Params = Params;
    type Error = ToolError;

    fn name(&self) -> &'static str {
        "ListFiles"
    }

    fn description(&self) -> &'static str {
        "Lists files in a folder"
    }

    async fn call(&mut self, parameters: Self::Params) -> Result<String, Self::Error> {
        let path = parameters.path.as_deref().unwrap_or(".");
        let show_hidden = parameters.show_hidden;

        let mut args = vec!["-1"];
        if show_hidden {
            args.push("-A");
        }
        args.push(path);

        let output = tokio::process::Command::new("ls")
            .args(&args)
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            Ok(format!("ls failed: {}", stderr))
        }
    }
}
