use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PistonVersion {
    id: String,
    #[serde(rename = "type")]
    kind: Kind,
    url: Url,
    time: DateTime<Utc>,
    release_time: DateTime<Utc>,
    sha1: String,
    compliance_level: u32,
}

impl PistonVersion {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn kind(&self) -> Kind {
        self.kind
    }
    pub fn url(&self) -> &Url {
        &self.url
    }
    pub fn time(&self) -> &DateTime<Utc> {
        &self.time
    }
    pub fn release_time(&self) -> &DateTime<Utc> {
        &self.release_time
    }
    pub fn sha1(&self) -> &str {
        &self.sha1
    }
    pub fn compliance_level(&self) -> u32 {
        self.compliance_level
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot")]
    Snapshot,
    #[serde(rename = "oldBeta")]
    OldBeta,
    #[serde(rename = "oldAlpha")]
    OldAlpha,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Release => "release",
            Kind::Snapshot => "snapshot",
            Kind::OldBeta => "oldBeta",
            Kind::OldAlpha => "oldAlpha",
        }
    }
}
