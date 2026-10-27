use std::path::Path;
use crate::core::client::FabricClient;
use crate::core::errors::SdkError;
use crate::core::launch::{LaunchConfig, LaunchConfigBuilder};
use crate::core::verifier::AuthSession;
use crate::loaders::piston::models::version::Kind;

impl FabricClient {
    pub async fn prepare_vanilla_launch(
        &self,
        session: AuthSession,
        version_id: impl Into<String>,
        version_type: Kind,
        asset_index: impl Into<String>,
        game_dir: impl AsRef<Path> + Send + Sync + 'static,
        classpath: Vec<impl AsRef<Path> + Send + Sync + 'static>,
        java_path: impl AsRef<Path> + Send + Sync + 'static,
    ) -> Result<LaunchConfigBuilder, SdkError> {


        let builder = LaunchConfig::builder()
            .main_class("net.fabricmc.loader.launch.knot.KnotClient");

        Ok(builder)
    }
}
