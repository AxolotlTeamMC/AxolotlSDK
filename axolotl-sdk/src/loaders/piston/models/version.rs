use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Debug)]
pub struct PistonVersion {
    id: String,
    #[serde(rename = "type")]
    kind: Kind,
    url: Url,
    time: DateTime<Utc>,
    #[serde(rename = "releaseTime")]
    release_time: DateTime<Utc>,
    sha1: String,
    #[serde(rename = "complianceLevel")]
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

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Release => "release",
            Kind::Snapshot => {
                "snapshot"
            }
            Kind::OldBeta => {
                "oldBeta"
            }
            Kind::OldAlpha => {
                "oldAlpha"
            }
        }
    }
}
