use crate::core::client::{AuthSession, PistonClient};
use crate::core::launch::{LaunchConfig, LaunchConfigBuilder};
use crate::core::errors::SdkError;
use crate::loaders::piston::models::version_details::PistonVersionDetails;
use std::path::{Path, PathBuf};
use crate::loaders::piston::models::version::Kind;

impl PistonClient {
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
        let java_path = java_path.as_ref().to_path_buf();
        let game_dir = game_dir.as_ref().to_path_buf();
        let classpath: Vec<PathBuf> = classpath
            .into_iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();

        let version_id = version_id.into();
        let asset_index_id = asset_index.into();

        let (username, uuid, access_token, user_type, xuid) = match session.clone() {
            AuthSession::Online { username, uuid, access_token, xuid } => {
                (username, uuid, access_token, session.resolve_user_type(&version_id), Some(xuid))
            }
            AuthSession::Offline { username, uuid } => {
                (username, uuid, String::new(), session.resolve_user_type(&version_id), None)
            }
        };

        let native_dir = game_dir.join("versions").join(&version_id).join("natives");

        let builder = LaunchConfig::builder()
            .java_path(java_path)
            .game_dir(game_dir)
            .version_id(version_id)
            .username(username)
            .uuid(uuid)
            .user_type(user_type)
            .classpath(classpath)
            .version_type(version_type)
            .asset_index_id(asset_index_id)
            .native_dir(native_dir)
            .access_token(access_token)
            .xuid(xuid)
            .memory_max_mb(2048);

        Ok(builder)
    }

    pub fn build_classpath<P>(&self, version: &PistonVersionDetails, libraries_dir: P) -> Vec<PathBuf>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        version
            .libraries()
            .iter()
            .filter(|lib| lib.should_download())
            .filter_map(|lib| lib.downloads().artifact())
            .map(|art| libraries_dir.as_ref().join(&art.path))
            .collect::<Vec<_>>()
    }
}
