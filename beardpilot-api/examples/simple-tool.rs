use beardpilot_api::{
    client::MistralClient,
    endpoint::{
        chat::{Chat, Message},
        tool::{ErasedTool, Tool},
    },
    error::EndpointError,
};
use schemars::JsonSchema;

struct ListFiles;

#[derive(serde::Deserialize, JsonSchema)]
pub struct ListFilesParams {
    path: Option<String>,
}

impl ListFiles {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for ListFiles {
    type Params = ListFilesParams;
    type Error = EndpointError;
    fn name(&self) -> &'static str {
        "list_files"
    }

    fn description(&self) -> &'static str {
        "List files in directory"
    }

    async fn call(&mut self, parameters: Self::Params) -> Result<String, Self::Error> {
        Ok("README.md\nCargo.toml\nsrc/\n".to_owned())
    }
}

#[tokio::main]
async fn main() -> Result<(), EndpointError> {
    let api_key = std::env::var("MISTRAL_API_KEY").expect("Env var MISTRAL_API_KEY must be set");
    let client = MistralClient::new("https://api.mistral.ai", &api_key)?;

    let messages = vec![
        Message::system("You are a helpful assistant"),
        Message::user("List files in current folder"),
    ];
    let tools: Vec<Box<dyn ErasedTool>> = vec![Box::new(ListFiles::new())];
    let chat = Chat::new("mistral-small-latest", messages)
        .with_tools(tools)
        .build();
    let response = client.chat(chat).await?;
    for tool_call in response.choices[0].message.tool_calls.iter().flatten() {
        println!("{:?}", tool_call);
    }

    println!("{}", response.content());

    Ok(())
}
