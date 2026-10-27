use serde::Deserialize;
use url::Url;
use crate::core::download::Downloadable;
use crate::loaders::piston::models::assets::AssetIndexInfo;
use crate::loaders::piston::models::libraries::{Library, Rule};

#[derive(Deserialize, Debug, Default, Clone)]
pub struct PistonVersionDetails {
    id: String,
    assets: String,

    #[serde(rename = "assetIndex")]
    asset_index: AssetIndexInfo,

    downloads: GameDownloads,

    #[serde(rename = "javaVersion")]
    java_version: JavaVersion,
    arguments: Option<Arguments>,
    libraries: Vec<Library>,
}

impl PistonVersionDetails {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn assets(&self) -> &str {
        &self.assets
    }

    pub fn asset_index(&self) -> &AssetIndexInfo {
        &self.asset_index
    }

    pub fn downloads(&self) -> &GameDownloads {
        &self.downloads
    }

    pub fn java_version(&self) -> &JavaVersion {
        &self.java_version
    }

    pub fn arguments(&self) -> &Option<Arguments> {
        &self.arguments
    }

    pub fn libraries(&self) -> &Vec<Library> {
        &self.libraries
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Arguments {
    pub game: Vec<ArgumentValue>,
    pub jvm: Vec<ArgumentValue>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ArgumentValue {
    Simple(String),
    Conditional(ConditionalArgument),
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConditionalArgument {
    pub rules: Vec<Rule>,
    pub value: ArgumentInnerValue,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ArgumentInnerValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    major_version: u32,
}

impl JavaVersion {
    pub fn major_version(&self) -> u32 {
        self.major_version
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct GameDownloads {
    client: DownloadInfo,
    #[serde(rename = "client_mappings")]
    client_mappings: Option<DownloadInfo>,
    server: Option<DownloadInfo>,
    #[serde(rename = "server_mappings")]
    server_mappings: Option<DownloadInfo>,
}

impl GameDownloads {
    pub fn client(&self) -> &DownloadInfo { &self.client }
    pub fn server(&self) -> Option<&DownloadInfo> { self.server.as_ref() }
    pub fn client_mappings(&self) -> Option<&DownloadInfo> { self.client_mappings.as_ref() }
    pub fn server_mappings(&self) -> Option<&DownloadInfo> { self.server_mappings.as_ref() }
}

#[derive(Deserialize, Debug, Clone)]
pub struct DownloadInfo {
    sha1: String,
    size: u64,
    url: Url,
}

impl Default for DownloadInfo {
    fn default() -> Self {
        Self {
            sha1: String::new(),
            size: 0,
            url: Url::parse("http://localhost").unwrap(),
        }
    }
}

impl Downloadable for DownloadInfo {
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

impl DownloadInfo {
    pub fn size(&self) -> u64 {
        self.size
    }
}