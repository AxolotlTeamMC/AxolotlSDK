use std::collections::HashMap;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::core::download::Downloadable;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexInfo {
    id: String,
    sha1: String,
    size: u64,
    total_size: u64,
    url: Url,
}

impl AssetIndexInfo {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn total_size(&self) -> u64 {
        self.total_size
    }
}

#[derive(Deserialize, Debug)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Deserialize, Debug)]
pub struct AssetObject {
    hash: String,
    size: u64,
}

impl AssetObject {
    pub fn hash(&self) -> &str {
        &self.hash
    }
    pub fn size(&self) -> u64 {
        self.size
    }
}

impl Downloadable for AssetObject {
    fn url(&self) -> Url {
        static BASE_URL: OnceLock<Url> = OnceLock::new();

        let prefix = self.hash.get(0..2).unwrap_or("xx");

        let mut url = BASE_URL
            .get_or_init(|| Url::parse("https://resources.download.minecraft.net/").unwrap())
            .clone();

        if let Ok(mut segments) = url.path_segments_mut() {
            segments.push(prefix).push(&self.hash);
        }

        url
    }

    fn sha1(&self) -> &str {
        &self.hash
    }

    fn size(&self) -> u64 {
        self.size
    }
}

impl Downloadable for AssetIndexInfo {
    fn url(&self) -> Url {
        self.url.clone()
    }

    fn sha1(&self) -> &str {
        &self.sha1
    }

    fn size(&self) -> u64 {
        self.size
    }
}
