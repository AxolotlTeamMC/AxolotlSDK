use std::collections::HashMap;
use serde::Deserialize;
use url::Url;
use crate::core::download::Downloadable;

#[derive(Deserialize, Debug, Clone)]
pub struct Library {
    name: String,
    downloads: LibraryDownloads,
    rules: Option<Vec<Rule>>,
    natives: Option<NativesConfig>,
    extract: Option<ExtractConfig>,
}

impl Library {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn downloads(&self) -> &LibraryDownloads {
        &self.downloads
    }

    pub fn rules(&self) -> &Option<Vec<Rule>> {
        &self.rules
    }

    pub fn natives(&self) -> &Option<NativesConfig> {
        &self.natives
    }

    pub fn extract(&self) -> Option<&ExtractConfig> {
        self.extract.as_ref()
    }

    pub fn get_native_classifier(&self) -> Option<&str> {
        if !self.should_download() {
            return None;
        }
        self.natives.as_ref()?.get_current_classifier()
    }


    pub fn should_download(&self) -> bool {
        Self::evaluate_rules(self.rules())
    }

    pub fn evaluate_rules(rules: &Option<Vec<Rule>>) -> bool {
        let rules = match rules {
            Some(rules) => rules,
            None => return true,
        };

        let current_os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "linux"
        };

        let mut allowed = false;

        for rule in rules {
            let os_matches = match &rule.os {
                Some(os_restrict) => match &os_restrict.name {
                    Some(os_name) => os_name == current_os,
                    None => true,
                },
                None => true,
            };

            if os_matches {
                if rule.action == "allow" {
                    allowed = true;
                } else if rule.action == "disallow" {
                    allowed = false;
                }
            }
        }

        allowed
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExtractConfig {
    pub exclude: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LibraryDownloads {
    artifact: Option<Artifact>,
    classifiers: Option<HashMap<String, Artifact>>,
}

impl LibraryDownloads {
    pub fn artifact(&self) -> Option<&Artifact> {
        self.artifact.as_ref()
    }

    pub fn classifiers(&self) -> Option<&HashMap<String, Artifact>> {
        self.classifiers.as_ref()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Artifact {
    pub path: String,
    sha1: String,
    size: u64,
    url: Url,
}

impl Downloadable for Artifact {
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

#[derive(Deserialize, Debug, Clone)]
pub struct Rule {
    action: String,
    os: Option<OsRestriction>,
    features: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OsRestriction {
    name: Option<String>,
    arch: Option<String>,
    version: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NativesConfig {
    #[serde(rename = "windows")]
    pub windows: Option<String>,
    #[serde(rename = "linux")]
    pub linux: Option<String>,
    #[serde(rename = "osx")]
    pub osx: Option<String>,
}

impl NativesConfig {
    pub fn get_current_classifier(&self) -> Option<&str> {
        if cfg!(target_os = "windows") {
            self.windows.as_deref()
        } else if cfg!(target_os = "macos") {
            self.osx.as_deref()
        } else {
            self.linux.as_deref()
        }
    }
}