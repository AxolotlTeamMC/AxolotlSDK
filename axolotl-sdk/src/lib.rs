use serde::Deserialize;

pub mod core;
pub mod loaders;
pub mod models;

#[derive(Deserialize)]
pub enum FetchResult<T> {
    Fetched { data: T, etag: Option<String> },
    NotModified,
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub fn get_mojang_os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}