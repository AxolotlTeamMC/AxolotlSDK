use serde::Deserialize;

pub type FabricLoaderList = Vec<FabricLoader>;

#[derive(Deserialize, Debug)]
pub struct FabricLoader {
    version: String,
    stable: bool,
    maven: String,
    build: u32,
    separator: String,
}

impl FabricLoader {
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn stable(&self) -> bool {
        self.stable
    }

    pub fn maven(&self) -> &str {
        &self.maven
    }

    pub fn build(&self) -> u32 {
        self.build
    }

    pub fn separator(&self) -> &str {
        &self.separator
    }
}