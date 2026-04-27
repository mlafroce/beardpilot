#[cfg(feature = "stream")]
use futures_util::Stream;
use std::fmt::Debug;
use url::Url;

use crate::endpoint::{
    chat::{Chat, ChatResponse},
    embed::{Embed, EmbedResponse},
    generate::{Generate, GenerateResponse},
    model::ModelList,
    tag::TagList,
    version::Version,
};
use crate::error::{EndpointError, ProviderError};

#[derive(Debug)]
pub struct MistralClient {
    pub(crate) url: Url,
    pub(crate) reqwest_client: reqwest::Client,
    pub(crate) api_key: String,
}

impl MistralClient {
    pub fn new(host: &str, api_key: &str) -> Result<Self, EndpointError> {
        Ok(Self {
            url: Url::parse(&host)?,
            reqwest_client: reqwest::Client::new(),
            api_key: api_key.to_string(),
        })
    }

    async fn get_endpoint<Resp>(&self, endpoint: &str) -> Result<Resp, EndpointError>
    where
        Resp: serde::de::DeserializeOwned,
    {
        let response = self
            .reqwest_client
            .get(self.url.join(endpoint).unwrap())
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            .header("Authorization", format!("Bearer {}", self.api_key))
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
        use std::marker::PhantomData;

        use crate::endpoint::EndpointStream;

        let response = self
            .reqwest_client
            .post(self.url.join(endpoint).unwrap())
            .header("Authorization", format!("Bearer {}", self.api_key))
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

    /// Retrieve the version of the Mistral
    pub async fn get_version(&self) -> Result<Version, EndpointError> {
        self.get_endpoint("/api/version").await
    }

    /// Creates vector embeddings representing the input text
    pub async fn embed(&self, embed: Embed) -> Result<EmbedResponse, EndpointError> {
        self.post_endpoint("/api/embed", embed).await
    }

    /// Generate the next chat message in a conversation between a user and an assistant.
    pub async fn chat(&self, mut chat: Chat) -> Result<ChatResponse, EndpointError> {
        chat.stream = false;
        self.post_endpoint("/v1/chat/completions", chat).await
    }

    /// Generate the next chat message in a conversation between a user and an assistant, in a stream.
    #[cfg(feature = "stream")]
    pub async fn chat_stream(
        &self,
        mut chat: Chat,
    ) -> Result<impl Stream<Item = Result<ChatResponse, EndpointError>>, EndpointError> {
        chat.stream = true;
        self.post_endpoint_stream("/v1/chat/completions", chat)
            .await
    }

    /// Generates a response for the provided prompt, in a stream
    pub async fn generate(
        &self,
        mut generate: Generate,
    ) -> Result<GenerateResponse, EndpointError> {
        generate.stream = Some(false);
        self.post_endpoint("/api/generate", generate).await
    }

    /// Generates a response for the provided prompt, in a stream
    #[cfg(feature = "stream")]
    pub async fn generate_stream(
        &self,
        mut generate: Generate,
    ) -> Result<impl Stream<Item = Result<GenerateResponse, EndpointError>>, EndpointError> {
        generate.stream = Some(true);
        self.post_endpoint_stream("/api/generate", generate).await
    }
}
