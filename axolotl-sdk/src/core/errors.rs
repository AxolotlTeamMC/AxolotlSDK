use thiserror::Error;
use tokio::sync::AcquireError;
use tokio::sync::mpsc::error::SendError;
use crate::core::progress::DownloadEvent;

#[derive(Error, Debug)]
pub enum SdkError {
    /// Ошибка сети (например, пропал интернет или сервер недоступен)
    #[error("сетевой запрос завершился ошибкой: {0}")]
    Network(#[from] reqwest::Error),

    /// Ошибка файловой системы (не удалось создать файл, нет прав, кончилось место)
    #[error("ошибка работы с диском (I/O): {0}")]
    Io(#[from] std::io::Error),

    /// Сервер вернул ошибку вместо файла или данных (например, 404 Not Found или 500)
    #[error("сервер вернул некорректный статус ответа: {0}")]
    BadStatus(reqwest::StatusCode),

    /// Данные скачались, но их хэш-сумма не совпала с заявленной
    #[error("контрольная сумма файла не совпала: ожидалось {expected}, получено {actual}")]
    ChecksumMismatch {
        expected: String,
        actual: String,
    },

    /// Сервер прислал битый JSON/Protobuf, который мы не смогли распарсить
    #[error("ошибка десериализации данных от сервера: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Переданная строка содержит символы, запрещенные в HTTP-заголовках
    #[error("некорректный формат User-Agent или токена авторизации")]
    InvalidHeaderValue,

    /// Ошибка парсинга URL-адреса (если пользователь передал Custom-урл стенда с ошибкой)
    #[error("некорректный базовый URL сервера: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("ошибка семафора: {0}")]
    SemaphoreError(#[from] AcquireError),

    #[error("ошибка не нахождения пути")]
    InvalidPath,

    #[error("ошибка скачивания: {0}")]
    DownloadError(String),

    #[error("Failed to send download event: {0}")]
    SendError(#[from] SendError<DownloadEvent>),

    #[error("Failed to download file '{0}' after 3 attempts.")]
    MaxAttemptsExceeded(String),

    #[error("Error configure builder: {0}")]
    BuilderError(String),

    #[error("Error extract: {0}")]
    ExtractionError(String),
}