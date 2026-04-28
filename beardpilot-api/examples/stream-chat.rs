use beardpilot_api::{
    client::MistralClient,
    endpoint::chat::{Chat, Message},
    error::EndpointError,
};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), EndpointError> {
    let api_key = std::env::var("MISTRAL_API_KEY").expect("Env var MISTRAL_API_KEY must be set");
    let client = MistralClient::new("https://api.mistral.ai", &api_key)?;

    let messages = vec![
        Message::system("You are a helpful assistant"),
        Message::user("Explain what a coding agent is"),
    ];
    let chat = Chat::new("mistral-small-latest", messages).build();
    let mut stream = client.chat_stream(chat).await?;

    while let Some(chunk) = stream.next().await {
        print!("{}", chunk.expect("Chunk parse failed").content());
    }

    Ok(())
}
