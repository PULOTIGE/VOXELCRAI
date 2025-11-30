use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningFile {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub file_type: FileType,
    pub size: u64,
    pub uploaded_at: f64,
    pub metadata: LearningMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Text,
    Video,
    Image,
    Audio,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearningMetadata {
    pub duration: Option<f64>, // Для видео/аудио
    pub resolution: Option<(u32, u32)>, // Для видео/изображений
    pub encoding: Option<String>,
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

pub struct LearningMode {
    files: Vec<LearningFile>,
    upload_directory: PathBuf,
    max_file_size: u64, // в байтах
    supported_text_formats: Vec<String>,
    supported_video_formats: Vec<String>,
}

impl LearningMode {
    pub fn new() -> Self {
        let upload_dir = std::env::temp_dir().join("adaptive_engine_uploads");
        
        // Создаем директорию для загрузок, если её нет
        if !upload_dir.exists() {
            let _ = fs::create_dir_all(&upload_dir);
        }

        Self {
            files: Vec::new(),
            upload_directory: upload_dir,
            max_file_size: 500 * 1024 * 1024, // 500 МБ по умолчанию
            supported_text_formats: vec![
                "txt".to_string(),
                "md".to_string(),
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "rtf".to_string(),
            ],
            supported_video_formats: vec![
                "mp4".to_string(),
                "avi".to_string(),
                "mov".to_string(),
                "mkv".to_string(),
                "webm".to_string(),
                "flv".to_string(),
            ],
        }
    }

    pub fn detect_file_type(path: &Path) -> FileType {
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            
            match ext_lower.as_str() {
                "txt" | "md" | "pdf" | "doc" | "docx" | "rtf" => FileType::Text,
                "mp4" | "avi" | "mov" | "mkv" | "webm" | "flv" => FileType::Video,
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => FileType::Image,
                "mp3" | "wav" | "ogg" | "flac" | "aac" => FileType::Audio,
                other => FileType::Other(other.to_string()),
            }
        } else {
            FileType::Other("unknown".to_string())
        }
    }

    pub fn upload_file(&mut self, source_path: &Path) -> Result<String, String> {
        // Проверка существования файла
        if !source_path.exists() {
            return Err("Файл не найден".to_string());
        }

        // Проверка размера
        let metadata = fs::metadata(source_path)
            .map_err(|e| format!("Ошибка чтения метаданных: {}", e))?;
        
        if metadata.len() > self.max_file_size {
            return Err(format!(
                "Файл слишком большой. Максимальный размер: {} МБ",
                self.max_file_size / (1024 * 1024)
            ));
        }

        // Копируем файл в директорию загрузок
        let file_name = source_path
            .file_name()
            .ok_or("Неверное имя файла")?
            .to_string_lossy()
            .to_string();
        
        let dest_path = self.upload_directory.join(&file_name);
        
        // Если файл уже существует, добавляем суффикс
        let mut final_path = dest_path.clone();
        let mut counter = 1;
        while final_path.exists() {
            let stem = source_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = source_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            final_path = self.upload_directory.join(format!("{}_{}.{}", stem, counter, ext));
            counter += 1;
        }

        fs::copy(source_path, &final_path)
            .map_err(|e| format!("Ошибка копирования файла: {}", e))?;

        // Создаем запись о файле
        let file_type = Self::detect_file_type(&final_path);
        let file_id = uuid::Uuid::new_v4().to_string();
        
        let learning_file = LearningFile {
            id: file_id.clone(),
            name: file_name,
            path: final_path,
            file_type: file_type.clone(),
            size: metadata.len(),
            uploaded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            metadata: LearningMetadata::default(),
        };

        self.files.push(learning_file);
        Ok(file_id)
    }

    pub fn read_text_file(&self, file_id: &str) -> Result<String, String> {
        let file = self.files
            .iter()
            .find(|f| f.id == file_id)
            .ok_or("Файл не найден")?;

        match file.file_type {
            FileType::Text => {
                let mut content = String::new();
                fs::File::open(&file.path)
                    .and_then(|mut f| f.read_to_string(&mut content))
                    .map_err(|e| format!("Ошибка чтения файла: {}", e))?;
                Ok(content)
            }
            _ => Err("Файл не является текстовым".to_string()),
        }
    }

    pub fn get_file_info(&self, file_id: &str) -> Option<&LearningFile> {
        self.files.iter().find(|f| f.id == file_id)
    }

    pub fn list_files(&self) -> &[LearningFile] {
        &self.files
    }

    pub fn remove_file(&mut self, file_id: &str) -> Result<(), String> {
        let index = self.files
            .iter()
            .position(|f| f.id == file_id)
            .ok_or("Файл не найден")?;

        let file = &self.files[index];
        
        // Удаляем файл с диска
        if file.path.exists() {
            fs::remove_file(&file.path)
                .map_err(|e| format!("Ошибка удаления файла: {}", e))?;
        }

        self.files.remove(index);
        Ok(())
    }

    pub fn get_upload_directory(&self) -> &Path {
        &self.upload_directory
    }

    pub fn set_max_file_size(&mut self, size_mb: u64) {
        self.max_file_size = size_mb * 1024 * 1024;
    }
}

impl Default for LearningMode {
    fn default() -> Self {
        Self::new()
    }
}
