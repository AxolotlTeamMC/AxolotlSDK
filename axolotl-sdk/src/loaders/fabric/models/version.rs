use serde::Deserialize;
pub type FabricVersionList = Vec<FabricVersion>;

#[derive(Deserialize, Debug)]
pub struct FabricVersion {
    pub version: String,
    pub stable: bool,
}

impl FabricVersion {
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn stable(&self) -> bool {
        self.stable
    }
}