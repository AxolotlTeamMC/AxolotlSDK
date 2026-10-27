use std::fs;
use axolotl_sdk::FetchResult::Fetched;
use directories::ProjectDirs;
use std::path::Path;
use std::process::Stdio;
use tokio::sync::mpsc;
use axolotl_sdk::core::client::{AuthSession, MCLClient};
use axolotl_sdk::core::errors::SdkError;
use axolotl_sdk::core::progress::DownloadEvent;

pub fn check_file<P>(file_path: P) -> Result<(), SdkError>
where
    P: AsRef<Path>,
{
    let path = file_path.as_ref();

    if !path.exists() || !path.is_file() {
        return Err(SdkError::InvalidPath);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let proj_dirs = ProjectDirs::from("com", "AxolotlTeam", "AxolotlLauncher")
        .expect("Не удалось определить системные директории пользователя!");

    let client = MCLClient::builder()
        .user_agent("DemoLaunch/1.0")
        .pool_size(16)
        .build();

    let session = AuthSession::offline("DemoName");

    let manifest = client.piston.fetch_manifest(None).await.unwrap();
    let Fetched {
        data: ok_manifest, ..
    } = manifest
    else {
        panic!("Манифест не был обновлен или недоступен!");
    };

    let latest = &ok_manifest.latest();
    // let version = &ok_manifest.find_version("1.16.5").unwrap();
    let version = &ok_manifest.find_version(&latest.release).unwrap();
    let version_details = client.piston.fetch_version_details(version).await.unwrap();

    let base_dir = proj_dirs.data_dir().to_path_buf();

    let client_dir = base_dir.join("client");
    let game_dir = base_dir.join("games");
    let assets_dir = base_dir.join("assets");
    let assets_indexes_dir = assets_dir.join("indexes");
    let libraries_dir = base_dir.join("libraries");

    // Создаем директорию для нативных библиотек (.dll) конкретной версии
    let natives_dir = base_dir.join("versions").join(version.id()).join("natives");

    fs::create_dir_all(&client_dir).unwrap();
    fs::create_dir_all(&game_dir).unwrap();
    fs::create_dir_all(&assets_dir).unwrap();
    fs::create_dir_all(&assets_indexes_dir).unwrap();
    fs::create_dir_all(&libraries_dir).unwrap();
    fs::create_dir_all(&natives_dir).unwrap();

    let java_path = r"C:\Users\MSI\.jdks\openjdk-25.0.1\bin\java.exe";

    let (tx, mut rx) = mpsc::channel::<DownloadEvent>(100);

    let logger_handle = tokio::spawn(async move {
        let mut total_bytes: usize = 0;
        while let Some(message) = rx.recv().await {
            if let DownloadEvent::FileChunk { bytes_written, .. } = &message {
                total_bytes += bytes_written;
            }
            // println!("💬 [Всего скачано: {} байт] | {:?}", total_bytes, message);
        }
    });

    let client_jar_path = client_dir.join(format!("{}-client.jar", version.id()));

    if check_file(&client_jar_path).is_ok() {
        let _ = tx
            .send(DownloadEvent::FileFinished {
                name: format!("{}.jar", version.id()),
            })
            .await;
    } else {
        println!("⏳ Файл клиента не найден. Скачиваем...");
        client
            .piston
            .download_client(&version_details, client_dir.clone(), tx.clone())
            .await
            .expect("Ошибка при скачивании клиента");
    }

    let asset_index_path =
        assets_indexes_dir.join(format!("{}.json", version_details.asset_index().id()));

    if check_file(&asset_index_path).is_ok() {
        let _ = tx
            .send(DownloadEvent::FileFinished {
                name: format!("{}.json", version_details.asset_index().id()),
            })
            .await;
    } else {
        println!("⏳ Индекс ассетов не найден. Скачиваем...");
        client
            .piston
            .download_asset_index(&version_details, assets_dir.clone(), tx.clone())
            .await
            .expect("Ошибка при скачивании ассет-индекса");
    }

    println!("⏳ Скачивание объектов ассетов...");
    client
        .piston
        .download_asset_objects(&version_details, assets_dir.clone(), tx.clone())
        .await
        .expect("Ошибка при скачивании объектов ассетов");

    println!("⏳ Проверка и скачивание библиотек...");
    client
        .piston
        .download_libraries(&version_details, libraries_dir.clone(), tx.clone())
        .await
        .expect("Ошибка при скачивании библиотек");

    println!("⏳ Скачивание и распаковка нативных библиотек...");
    client
        .piston
        .download_and_extract_natives(&version_details, natives_dir.clone(), tx.clone())
        .await
        .expect("Ошибка при скачивании и распаковке natives");

    let classpath = client
        .piston
        .build_classpath(&version_details, libraries_dir);

    let config_builder = client
        .piston
        .prepare_vanilla_launch(
            session,
            version.id(),
            version.kind(),
            version_details.asset_index().id(),
            game_dir,
            classpath,
            java_path,
        )
        .await
        .unwrap();

    drop(tx);
    let _ = logger_handle.await;

    let config = config_builder
        .memory_max_mb(4096)
        .client(client_jar_path)
        .assets_dir(assets_dir)
        .asset_index_id(version_details.asset_index().id())
        .native_dir(natives_dir)
        .build()
        .unwrap();

    let mut cmd = config.execute().unwrap();

    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    println!("🚀 Launch Minecraft {}...", version.id());

    let mut child = cmd.spawn().unwrap();

    child.wait().await.expect("Error");
}