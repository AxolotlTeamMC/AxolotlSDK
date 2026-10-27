use crate::FetchResult;
use crate::FetchResult::{Fetched, NotModified};
use crate::core::client::PistonClient;
use crate::loaders::piston::models::manifest::{PistonManifest};
use crate::loaders::piston::models::version::{PistonVersion};
use crate::core::download::Downloadable;
use crate::core::errors::SdkError;
use crate::loaders::piston::models::assets::{AssetIndex, AssetObject};
use crate::loaders::piston::models::version_details::PistonVersionDetails;
use reqwest::StatusCode;
use reqwest::header::{ETAG, IF_NONE_MATCH};
use std::collections::HashMap;
use url::Url;

impl PistonClient {
    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: Url) -> Result<T, SdkError> {
        let _permit = self
            .limiter
            .acquire()
            .await
            .map_err(|e| SdkError::SemaphoreError(e))?;

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(SdkError::BadStatus(response.status()));
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }

    pub async fn fetch_manifest(
        &self,
        cached_etag: Option<&str>,
    ) -> Result<FetchResult<PistonManifest>, SdkError> {
        let _permit = self
            .limiter
            .acquire()
            .await
            .map_err(|e| SdkError::SemaphoreError(e))?;

        let endpoint = self.base_url.join("mc/game/version_manifest_v2.json")?;
        let mut request = self.client.get(endpoint);

        if let Some(cached_etag) = cached_etag {
            request = request.header(IF_NONE_MATCH, cached_etag);
        }

        let response = request.send().await?;

        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(NotModified);
        }

        if !response.status().is_success() {
            return Err(SdkError::BadStatus(response.status()));
        }

        let new_etag = response
            .headers()
            .get(ETAG)
            .and_then(|etag| etag.to_str().ok())
            .map(|etag| etag.to_string());

        let data = response.json().await?;

        Ok(Fetched {
            data,
            etag: new_etag,
        })
    }

    pub async fn fetch_version_details(
        &self,
        version: &PistonVersion,
    ) -> Result<PistonVersionDetails, SdkError> {
        self.get_json(version.url().clone()).await
    }

    pub async fn fetch_asset_index(
        &self,
        version: &PistonVersionDetails,
    ) -> Result<HashMap<String, AssetObject>, SdkError> {
        let index: AssetIndex = self.get_json(version.asset_index().url().clone()).await?;
        
        Ok(index.objects)
    }
}
