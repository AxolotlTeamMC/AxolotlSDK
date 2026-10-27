use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::core::download::Downloadable;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

    pub fn rules(&self) -> Option<&[Rule]> {
        self.rules.as_deref()
    }

    pub fn natives(&self) -> Option<&NativesConfig> {
        self.natives.as_ref()
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

    pub fn evaluate_rules(rules: Option<&[Rule]>) -> bool {
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
            let os_matches = rule.os.as_ref().map_or(true, |os_restrict| {
                os_restrict.name.as_ref().map_or(true, |os_name| {
                    os_name == current_os || (current_os == "osx" && os_name == "macos")
                })
            });

            let features_match = rule.features.as_ref().map_or(true, |features| {
                features.as_object().map_or(true, |obj| obj.is_empty())
            });

            if os_matches && features_match {
                match rule.action.as_str() {
                    "allow" => allowed = true,
                    "disallow" => allowed = false,
                    _ => {}
                }
            }
        }

        allowed
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ExtractConfig {
    exclude: Option<Vec<String>>,
}

impl ExtractConfig {
    pub fn exclude(&self) -> Option<&[String]> {
        self.exclude.as_deref()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    path: String,
    sha1: String,
    size: u64,
    url: Url,
}

impl Artifact {
    pub fn path(&self) -> &str {
        &self.path
    }
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    action: String,
    os: Option<OsRestriction>,
    features: Option<serde_json::Value>,
}

impl Rule {
    pub fn action(&self) -> &str {
        &self.action
    }
    pub fn os(&self) -> Option<&OsRestriction> {
        self.os.as_ref()
    }
    pub fn features(&self) -> Option<&serde_json::Value> {
        self.features.as_ref()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OsRestriction {
    name: Option<String>,
    arch: Option<String>,
    version: Option<String>,
}

impl OsRestriction {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub fn arch(&self) -> Option<&str> {
        self.arch.as_deref()
    }
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NativesConfig {
    #[serde(rename = "windows")]
    windows: Option<String>,
    #[serde(rename = "linux")]
    linux: Option<String>,
    #[serde(rename = "osx")]
    osx: Option<String>,
}

impl NativesConfig {
    pub fn windows(&self) -> Option<&str> {
        self.windows.as_deref()
    }
    pub fn linux(&self) -> Option<&str> {
        self.linux.as_deref()
    }
    pub fn osx(&self) -> Option<&str> {
        self.osx.as_deref()
    }

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