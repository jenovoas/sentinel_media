use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoFile {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: u64,
    pub duration_sec: Option<u64>, // TODO: Leer metadata real
}

pub fn scan_directory(path: &Path) -> Result<Vec<VideoFile>, String> {
    let mut files = Vec::new();

    if !path.exists() {
        return Ok(files); // Retornar vacío si la carpeta no existe aún
    }

    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    if ext_str == "mp4" || ext_str == "mov" || ext_str == "mkv" {
                        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                        let created = metadata
                            .created()
                            .unwrap_or(SystemTime::now())
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        files.push(VideoFile {
                            name: path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            path: path.to_string_lossy().to_string(),
                            size_bytes: metadata.len(),
                            created_at: created,
                            duration_sec: None, // Placeholder para futura implementación con ffprobe
                        });
                    }
                }
            }
        }
    }

    // Ordenar por fecha de creación (más reciente primero)
    files.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(files)
}
