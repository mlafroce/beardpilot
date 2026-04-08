use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(
        description = "Path folder to list. Defaults to current directory if not provided."
    )]
    path: Option<String>,
    #[schemars(description = "Show hidden files")]
    show_hidden: Option<bool>,
}

#[derive(Default)]
pub struct ListFiles {}

impl Tool for ListFiles {
    type Params = Params;

    fn name() -> &'static str {
        "ListFiles"
    }

    fn description() -> &'static str {
        "Lists files in a folder"
    }

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let path = parameters.path.as_deref().unwrap_or(".");
        let show_hidden = parameters.show_hidden.unwrap_or(false);

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
            Ok(format!("ls failed: {}", stderr).into())
        }
    }
}
