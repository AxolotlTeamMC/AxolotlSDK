use serde::{Deserialize, Serialize};
use crate::loaders::piston::models::version::{Kind, Latest, PistonVersion};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PistonManifest {
    latest: Latest,
    versions: Vec<PistonVersion>,
}

impl PistonManifest {
    pub fn latest(&self) -> &Latest {
        &self.latest
    }

    pub fn versions(&self) -> &[PistonVersion] {
        &self.versions
    }

    pub fn find_version(&self, id: &str) -> Option<&PistonVersion> {
        self.versions.iter().find(|v| v.id() == id)
    }

    pub fn get_latest_kind(&self, kind: Kind) -> Option<&PistonVersion> {
        self.versions.iter().find(|v| v.kind() == kind)
    }
}