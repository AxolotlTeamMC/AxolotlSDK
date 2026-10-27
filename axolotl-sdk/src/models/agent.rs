use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Порт для IPC-сокета (связь с Rust)
    pub ipc_port: u16,

    /// Включить ли подробные логи агента в консоли
    pub enable_logging: bool,

    /// Кастомный путь к файлу маппингов (если понадобится на будущее)
    pub mappings_file: Option<PathBuf>,
}

impl AgentConfig {
    pub(crate) fn to_jvm_arg(&self) -> PathBuf {
        let new_path_buf = PathBuf::new();
        new_path_buf
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            ipc_port: 35565,
            enable_logging: true,
            mappings_file: None,
        }
    }
}
