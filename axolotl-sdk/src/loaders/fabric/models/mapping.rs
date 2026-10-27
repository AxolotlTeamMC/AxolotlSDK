use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct FabricMapping {
    pub game_version: String,
    pub separator: i32,
    pub build: i32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

impl FabricMapping {
    pub fn game_version(&self) -> &str {
        &self.game_version
    }
    pub fn separator(&self) -> i32 {
        self.separator
    }
    pub fn build(&self) -> i32 {
        self.build
    }
    pub fn maven(&self) -> &str {
        &self.maven
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn stable(&self) -> bool {
        self.stable
    }
}