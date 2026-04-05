use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "Expression to find in folder or file. Accepts regex.")]
    expression: String,
    #[schemars(
        description = "Path to the file or folder to search in. Defaults to current directory if not provided."
    )]
    path: Option<String>,
}

#[derive(Default)]
pub struct Find {}

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

        // Ask the user for confirmation before running grep
        let mut stdout = tokio::io::stdout();
        stdout
            .write_all(
                format!(
                    "\n[confirm] Run Find: grep -rn {:?} in {:?}? [y/N] ",
                    parameters.expression, search_path
                )
                .as_bytes(),
            )
            .await?;
        stdout.flush().await?;

        let mut line = String::new();
        tokio::io::BufReader::new(tokio::io::stdin())
            .read_line(&mut line)
            .await?;

        if !matches!(line.trim(), "y" | "Y") {
            return Ok("Operation cancelled by user.".to_string());
        }

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
            Err(format!("grep failed: {}", stderr).into())
        }
    }
}
