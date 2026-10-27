use crate::core::launch::UserType;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum AuthSession {
    Online {
        username: String,
        uuid: String,
        access_token: String,
        xuid: String,
    },
    Offline {
        username: String,
        uuid: String,
    },
}

impl AuthSession {
    pub fn offline(username: impl Into<String>) -> Self {
        let username = username.into();

        let data = format!("OfflinePlayer:{}", username);
        let uuid = Uuid::new_v3(&Uuid::nil(), data.as_bytes()).to_string();

        Self::Offline { username, uuid }
    }

    pub fn online(
        username: impl Into<String>,
        uuid: impl Into<String>,
        access_token: impl Into<String>,
        xuid: impl Into<String>,
    ) -> Self {
        Self::Online {
            username: username.into(),
            uuid: uuid.into(),
            access_token: access_token.into(),
            xuid: xuid.into(),
        }
    }

    pub fn resolve_user_type(&self, version_id: &str) -> UserType {
        match self {
            AuthSession::Online { .. } => UserType::Msa,

            AuthSession::Offline { .. } => {
                if Self::is_legacy_version(version_id) {
                    UserType::Legacy
                } else {
                    UserType::Offline
                }
            }
        }
    }

    fn is_legacy_version(version_id: &str) -> bool {
        let v = version_id.to_lowercase();

        if v.starts_with('b') || v.starts_with('a') || v.starts_with('c') || v.starts_with("rd-") {
            return true;
        }

        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() >= 2 {
            if let Ok(major) = parts[0].parse::<i32>() {
                if let Ok(minor) = parts[1].parse::<i32>() {
                    if major == 1 && minor < 6 {
                        return true;
                    }
                }
            }
        }

        false
    }
}
