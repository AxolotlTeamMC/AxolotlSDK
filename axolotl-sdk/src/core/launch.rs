use crate::models::agent::AgentConfig;
use crate::core::errors::SdkError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use crate::loaders::piston::models::version::Kind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserType {
    Mojang,
    Msa,
    Legacy,
    Offline,
}

impl UserType {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserType::Mojang => "mojang",
            UserType::Msa => "msa",
            UserType::Legacy => "legacy",
            UserType::Offline => "offline",
        }
    }
}

pub struct LaunchConfig {
    pub java_path: PathBuf,
    pub memory_max_mb: u64,
    pub native_dir: PathBuf,
    pub classpath: Vec<PathBuf>,
    pub version_id: String,
    pub client: PathBuf,
    pub extra_jvm_args: Vec<String>,
    pub java_agent: Option<AgentConfig>,
    pub main_class: String,
    pub username: String,
    pub game_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub asset_index_id: String,
    pub uuid: String,
    pub xuid: Option<String>,
    pub access_token: Option<String>,
    pub user_type: UserType,
    pub version_type: Kind,
}

impl LaunchConfig {
    pub fn builder() -> LaunchConfigBuilder {
        LaunchConfigBuilder::default()
    }

    pub fn execute(&self) -> Result<Command, SdkError> {
        let mut cmd = Command::new(&self.java_path);

        let memory_part_one = "-Xmx";
        let memory_part_two = "M";
        let mut memory_full = String::with_capacity(
            memory_part_one.len() + memory_part_two.len() + self.memory_max_mb as usize,
        );
        memory_full.push_str(memory_part_one);
        memory_full.push_str(&self.memory_max_mb.to_string());
        memory_full.push_str(memory_part_two);

        let native_libs_part = "-Djava.library.path=";

        let mut native_libs =
            String::with_capacity(native_libs_part.len() + self.native_dir.to_string_lossy().len());
        native_libs.push_str(native_libs_part);
        native_libs.push_str(&self.native_dir.to_string_lossy());

        let mut full_classpath = self.classpath.clone();

        // В идеале имя должно передаваться как и путь
        if !self.client.is_file() {
            return Err(SdkError::InvalidPath);
        }

        full_classpath.push(self.client.clone());

        let separator = if cfg!(windows) { ";" } else { ":" };

        let classpath: String = full_classpath
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join(separator);

        cmd.arg(memory_full);
        cmd.arg(native_libs);
        cmd.arg("-cp").arg(classpath);

        if let Some(agent) = &self.java_agent {
            cmd.arg(agent.to_jvm_arg());
        }

        for arg in &self.extra_jvm_args {
            cmd.arg(arg);
        }

        cmd.arg(&self.main_class);

        cmd.arg("--username").arg(&self.username);
        cmd.arg("--version").arg(&self.version_id);
        cmd.arg("--gameDir").arg(&self.game_dir);
        cmd.arg("--assetsDir").arg(&self.assets_dir);
        cmd.arg("--assetIndex").arg(&self.asset_index_id);
        cmd.arg("--uuid").arg(&self.uuid);

        // if let Some(xuid) = &self.xuid {
        //     cmd.arg("--xuid").arg(xuid);
        // }
        if let Some(access_token) = &self.access_token {
            cmd.arg("--accessToken").arg(access_token);
        }

        cmd.arg("--userType").arg(&self.user_type.as_str());
        cmd.arg("--versionType").arg(&self.version_type.as_str());

        Ok(cmd)
    }
}

#[derive(Default)]
pub struct LaunchConfigBuilder {
    java_path: Option<PathBuf>,
    memory_max_mb: Option<u64>,
    native_dir: Option<PathBuf>,
    classpath: Option<Vec<PathBuf>>,
    version_id: Option<String>,
    client: Option<PathBuf>,
    extra_jvm_args: Vec<String>,
    java_agent: Option<AgentConfig>,
    main_class: Option<String>,
    username: Option<String>,
    game_dir: Option<PathBuf>,
    assets_dir: Option<PathBuf>,
    asset_index_id: Option<String>,
    uuid: Option<String>,
    xuid: Option<String>,
    access_token: Option<String>,
    user_type: Option<UserType>,
    version_type: Option<Kind>,
}

impl LaunchConfigBuilder {
    pub fn java_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.java_path = Some(path.into());
        self
    }

    pub fn game_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.game_dir = Some(dir.into());
        self
    }

    pub fn version_id<S: Into<String>>(mut self, id: S) -> Self {
        self.version_id = Some(id.into());
        self
    }

    pub fn username<S: Into<String>>(mut self, name: S) -> Self {
        self.username = Some(name.into());
        self
    }

    pub fn uuid<S: Into<String>>(mut self, id: S) -> Self {
        self.uuid = Some(id.into());
        self
    }

    pub fn xuid<S: Into<String>>(mut self, xuid: Option<S>) -> Self {
        self.xuid = xuid.map(|x| x.into());
        self
    }

    pub fn memory_max_mb(mut self, mb: u64) -> Self {
        self.memory_max_mb = Some(mb);
        self
    }

    pub fn native_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.native_dir = Some(dir.into());
        self
    }

    pub fn classpath(mut self, cp: Vec<PathBuf>) -> Self {
        self.classpath = Some(cp);
        self
    }

    pub fn client<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.client = Some(dir.into());
        self
    }

    pub fn extra_jvm_args(mut self, args: Vec<String>) -> Self {
        self.extra_jvm_args = args;
        self
    }

    pub fn java_agent(mut self, agent: AgentConfig) -> Self {
        self.java_agent = Some(agent);
        self
    }

    pub fn main_class<S: Into<String>>(mut self, class: S) -> Self {
        self.main_class = Some(class.into());
        self
    }

    // pub fn access_token<S: Into<String>>(mut self, token: S, user_type: UserType) -> Self {
    // self.user_type = Some(user_type); // Если есть токен, меняем тип пользователя (например на Msa)
    pub fn access_token<S: Into<String>>(mut self, token: S) -> Self {
        self.access_token = Some(token.into());
        self
    }

    pub fn user_type(mut self, user_type: UserType) -> Self {
        self.user_type = Some(user_type);
        self
    }

    pub fn version_type(mut self, kind: Kind) -> Self {
        self.version_type = Some(kind);
        self
    }

    pub fn assets_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.assets_dir = Some(dir.into());
        self
    }

    pub fn asset_index_id<S: Into<String>>(mut self, id: S) -> Self {
        self.asset_index_id = Some(id.into());
        self
    }

    pub fn build(self) -> Result<LaunchConfig, SdkError> {
        Ok(LaunchConfig {
            java_path: self
                .java_path
                .ok_or_else(|| SdkError::BuilderError("java_path missing".into()))?,
            memory_max_mb: self.memory_max_mb.unwrap_or(2048),
            native_dir: self
                .native_dir
                .ok_or_else(|| SdkError::BuilderError("native_dir missing".into()))?,
            classpath: self.classpath.unwrap_or_default(),
            version_id: self
                .version_id
                .ok_or_else(|| SdkError::BuilderError("version_id missing".into()))?,
            client: self
                .client
                .ok_or_else(|| SdkError::BuilderError("client file missing".into()))?,
            extra_jvm_args: self.extra_jvm_args,
            java_agent: self.java_agent,
            main_class: self
                .main_class
                .unwrap_or_else(|| "net.minecraft.client.main.Main".to_string()),
            username: self
                .username
                .ok_or_else(|| SdkError::BuilderError("username missing".into()))?,
            game_dir: self
                .game_dir
                .ok_or_else(|| SdkError::BuilderError("game_dir missing".into()))?,
            assets_dir: self
                .assets_dir
                .ok_or_else(|| SdkError::BuilderError("assets_dir missing".into()))?,
            asset_index_id: self
                .asset_index_id
                .ok_or_else(|| SdkError::BuilderError("asset_index_id missing".into()))?,
            uuid: self
                .uuid
                .ok_or_else(|| SdkError::BuilderError("uuid missing".into()))?,
            xuid: self.xuid,
            access_token: self.access_token,
            user_type: self.user_type.unwrap_or(UserType::Mojang),
            version_type: self
                .version_type
                .ok_or_else(|| SdkError::BuilderError("version_type missing".into()))?,
        })
    }
}
