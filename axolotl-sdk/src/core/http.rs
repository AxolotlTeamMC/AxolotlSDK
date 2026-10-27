use async_trait::async_trait;
use reqwest::{Client, RequestBuilder, Response};
use serde::Deserialize;
use tokio::sync::Semaphore;
use url::Url;
use crate::core::errors::SdkError;

#[derive(Deserialize)]
pub enum FetchResult<T> {
    Fetched { data: T, etag: Option<String> },
    NotModified,
}

#[async_trait]
pub trait HttpClientExt {
    fn client(&self) -> &Client;
    fn limiter(&self) -> &Semaphore;

    async fn send_request(&self, request: RequestBuilder) -> Result<Response, SdkError> {
        let _permit = self
            .limiter()
            .acquire()
            .await?;

        let response = request.send().await?;
        Ok(response)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: Url) -> Result<T, SdkError> {
        let req = self.client().get(url);
        let response = self.send_request(req).await?;

        if !response.status().is_success() {
            return Err(SdkError::BadStatus(response.status()));
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }
}

