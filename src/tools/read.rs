use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "Path to file")]
    path: String,
}

#[derive(Default)]
pub struct Read {}

impl Tool for Read {
    type Params = Params;

    fn name() -> &'static str {
        "Read"
    }

    fn description() -> &'static str {
        "Reads file content"
    }

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let output = tokio::process::Command::new("cat")
            .args([&parameters.path])
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            Ok(format!("cat failed: {}", stderr).into())
        }
    }
}
