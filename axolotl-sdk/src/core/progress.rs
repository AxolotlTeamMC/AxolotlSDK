#[derive(Debug, Clone)]
pub enum DownloadEvent {
    /// Запущен общий процесс: (всего_файлов)
    Started { total_files: usize },
    /// Началось скачивание конкретного файла: (имя, размер)
    FileStarted { name: String, size: u64 },
    /// Прогресс записи чанка (для плавной отрисовки полосы загрузки): (имя, байт_записано)
    FileChunk { name: String, bytes_written: usize },
    /// Файл успешно скачан и проверен: (имя)
    FileFinished { name: String },
    /// Ошибка при скачивании конкретного файла
    FileFailed { name: String, error: String },
    /// Весь процесс завершен
    Finished,
}

#[derive(Clone, Debug)]
pub enum ProgressMessage {
    /// Старт скачивания: (всего_файлов)
    Started(usize),
    /// Прогресс конкретного файла: (имя_файла, скачано_байт_в_этом_чанке)
    FileChunk {
        file_name: String,
        bytes_written: usize,
    },
    /// Файл успешно скачан и проверен: (имя_файла)
    FileFinished(String),
    /// Общая ошибка
    Error(String),
}