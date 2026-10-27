use crate::core::download::Downloadable;
use crate::loaders::piston::models::assets::AssetIndexInfo;
use crate::loaders::piston::models::libraries::{Library, Rule};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PistonVersionDetails {
    id: String,
    assets: String,
    asset_index: AssetIndexInfo,
    downloads: GameDownloads,
    java_version: JavaVersion,
    arguments: Option<Arguments>,
    libraries: Vec<Library>,
}

impl PistonVersionDetails {

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

    pub fn arguments(&self) -> Option<&Arguments> {
        self.arguments.as_ref()
    }

    pub fn libraries(&self) -> &[Library] {
        &self.libraries
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Arguments {
    game: Vec<ArgumentValue>,
    jvm: Vec<ArgumentValue>,
}

impl Default for Arguments {
    fn default() -> Self {
        Self {
            game: vec![],
            jvm: vec![],
        }
    }
}

impl Arguments {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn game(&self) -> &[ArgumentValue] {
        &self.game
    }
    pub fn jvm(&self) -> &[ArgumentValue] {
        &self.jvm
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ArgumentValue {
    Simple(String),
    Conditional(ConditionalArgument),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ConditionalArgument {
    pub rules: Vec<Rule>,
    pub value: ArgumentInnerValue,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ArgumentInnerValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    component: String,
    major_version: u32,
}

impl JavaVersion {
    pub fn component(&self) -> &str {
        &self.component
    }
    pub fn major_version(&self) -> u32 {
        self.major_version
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GameDownloads {
    client: DownloadInfo,
    #[serde(rename = "client_mappings")]
    client_mappings: Option<DownloadInfo>,
    server: Option<DownloadInfo>,
    #[serde(rename = "server_mappings")]
    server_mappings: Option<DownloadInfo>,
}

impl GameDownloads {
    pub fn client(&self) -> &DownloadInfo {
        &self.client
    }
    pub fn server(&self) -> Option<&DownloadInfo> {
        self.server.as_ref()
    }
    pub fn client_mappings(&self) -> Option<&DownloadInfo> {
        self.client_mappings.as_ref()
    }
    pub fn server_mappings(&self) -> Option<&DownloadInfo> {
        self.server_mappings.as_ref()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DownloadInfo {
    sha1: String,
    size: u64,
    url: Url,
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
