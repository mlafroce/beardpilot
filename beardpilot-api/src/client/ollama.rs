#[cfg(feature = "stream")]
use std::fmt::Debug;
use std::marker::PhantomData;

use futures_util::Stream;
use url::Url;

use crate::endpoint::{
    chat::{Chat, ChatStreamResponse},
    embed::{Embed, EmbedResponse},
    generate::{Generate, GenerateResponse},
    model::ModelList,
    tag::TagList,
    version::Version,
    EndpointStream,
};
use crate::error::{EndpointError, ProviderError};

#[derive(Debug)]
pub struct Ollama {
    pub(crate) url: Url,
    pub(crate) reqwest_client: reqwest::Client,
}

impl Default for Ollama {
    /// Returns a default Ollama instance with the host set to `http://127.0.0.1:11434`.
    fn default() -> Self {
        Self {
            url: Url::parse("http://127.0.0.1:11434").unwrap(),
            reqwest_client: reqwest::Client::new(),
        }
    }
}

impl Ollama {
    pub fn new(host: &str, port: u16) -> Result<Self, EndpointError> {
        let url_string = format!("http://{}:{}", host, port);
        Ok(Self {
            url: Url::parse(&url_string)?,
            reqwest_client: reqwest::Client::new(),
        })
    }

    async fn get_endpoint<Resp>(&self, endpoint: &str) -> Result<Resp, EndpointError>
    where
        Resp: serde::de::DeserializeOwned,
    {
        let response = self
            .reqwest_client
            .get(self.url.join(endpoint).unwrap())
            .send()
            .await?;
        let body = response.bytes().await?;
        if let Ok(error_resp) = serde_json::from_slice::<ProviderError>(&body) {
            return Err(EndpointError::ClientError(error_resp.error));
        }
        let response = serde_json::from_slice::<Resp>(&body)?;
        Ok(response)
    }

    async fn post_endpoint<Req, Resp>(
        &self,
        endpoint: &str,
        request: Req,
    ) -> Result<Resp, EndpointError>
    where
        Req: serde::ser::Serialize,
        Resp: serde::de::DeserializeOwned,
    {
        let response = self
            .reqwest_client
            .post(self.url.join(endpoint).unwrap())
            .json(&request)
            .send()
            .await?;
        let body = response.bytes().await?;
        if let Ok(error_resp) = serde_json::from_slice::<ProviderError>(&body) {
            return Err(EndpointError::ClientError(error_resp.error));
        }
        let response = serde_json::from_slice::<Resp>(&body)?;
        Ok(response)
    }

    #[cfg(feature = "stream")]
    async fn post_endpoint_stream<Req, Resp>(
        &self,
        endpoint: &str,
        request: Req,
    ) -> Result<impl Stream<Item = Result<Resp, EndpointError>>, EndpointError>
    where
        Req: serde::ser::Serialize,
        Resp: serde::de::DeserializeOwned + Unpin + Debug,
    {
        let response = self
            .reqwest_client
            .post(self.url.join(endpoint).unwrap())
            .json(&request)
            .send()
            .await?;

        let stream = response.bytes_stream();
        Ok(EndpointStream {
            stream,
            _data: PhantomData::<Resp>,
            buffer: String::new(),
        })
    }

    /// Retrieve a list of models that are currently running
    pub async fn get_ps(&self) -> Result<ModelList, EndpointError> {
        self.get_endpoint("/api/ps").await
    }

    /// Fetch a list of models and their details
    pub async fn get_tags(&self) -> Result<TagList, EndpointError> {
        self.get_endpoint("/api/tags").await
    }

    /// Retrieve the version of the Ollama
    pub async fn get_version(&self) -> Result<Version, EndpointError> {
        self.get_endpoint("/api/version").await
    }

    /// Creates vector embeddings representing the input text
    pub async fn post_embed(&self, embed: Embed) -> Result<EmbedResponse, EndpointError> {
        self.post_endpoint("/api/embed", embed).await
    }

    /// Generate the next chat message in a conversation between a user and an assistant.
    pub async fn post_chat(&self, mut chat: Chat) -> Result<ChatStreamResponse, EndpointError> {
        chat.stream = false;
        self.post_endpoint("/api/chat", chat).await
    }

    /// Generate the next chat message in a conversation between a user and an assistant, in a stream.
    #[cfg(feature = "stream")]
    pub async fn post_chat_stream(
        &self,
        mut chat: Chat,
    ) -> Result<impl Stream<Item = Result<ChatStreamResponse, EndpointError>>, EndpointError> {
        chat.stream = true;
        self.post_endpoint_stream("/api/chat", chat).await
    }

    /// Generates a response for the provided prompt, in a stream
    pub async fn post_generate(
        &self,
        mut generate: Generate,
    ) -> Result<GenerateResponse, EndpointError> {
        generate.stream = Some(false);
        self.post_endpoint("/api/generate", generate).await
    }

    /// Generates a response for the provided prompt, in a stream
    #[cfg(feature = "stream")]
    pub async fn post_generate_stream(
        &self,
        mut generate: Generate,
    ) -> Result<impl Stream<Item = Result<GenerateResponse, EndpointError>>, EndpointError> {
        generate.stream = Some(true);
        self.post_endpoint_stream("/api/generate", generate).await
    }
}
