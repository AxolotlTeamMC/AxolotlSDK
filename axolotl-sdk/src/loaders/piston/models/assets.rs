use std::collections::HashMap;
use serde::Deserialize;
use url::Url;
use crate::core::download::Downloadable;

#[derive(Deserialize, Debug, Clone)]
pub struct AssetIndexInfo {
    id: String,
    sha1: String,
    size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    url: Url,
}

impl Default for AssetIndexInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            sha1: String::new(),
            size: 0,
            total_size: 0,
            url: Url::parse("http://localhost").unwrap(),
        }
    }
}

impl AssetIndexInfo {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Deserialize, Debug)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Deserialize, Debug)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

impl Downloadable for AssetObject {
    fn url(&self) -> Url {
        let prefix = &self.hash[0..2];
        let sub_path = format!("{}/{}", prefix, self.hash);

        Url::parse("https://resources.download.minecraft.net/").unwrap().join(&sub_path).unwrap()
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
