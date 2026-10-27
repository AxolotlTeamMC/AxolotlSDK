use serde::{Deserialize, Serialize};
use crate::loaders::fabric::models::loaders::FabricLoader;

pub type VersionDetails = Vec<FabricLoaderDetails>;

#[derive(Deserialize, Debug)]
pub struct FabricLoaderDetails {
    loader: FabricLoader,
    intermediary: FabricIntermediaryInfo,
    #[serde(rename = "launcherMeta")]
    launcher_meta: FabricLauncherMeta,
}

impl FabricLoaderDetails {
    // pub fn loader(&self) -> FabricLoader {
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricIntermediaryInfo {
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricLauncherMeta {
    pub version: i32,
    #[serde(rename = "min_java_version")]
    pub min_java_version: i32,
    pub libraries: LibrariesCollection,
    #[serde(rename = "mainClass")]
    pub main_class: MainClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrariesCollection {
    #[serde(default)]
    pub client: Vec<LibraryItem>,
    #[serde(default)]
    pub common: Vec<LibraryItem>,
    #[serde(default)]
    pub server: Vec<LibraryItem>,
    #[serde(default)]
    pub development: Vec<LibraryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItem {
    pub name: String,
    pub url: String,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainClass {
    pub client: String,
    pub server: String,
}