use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuración para un canal de YouTube, leído desde `channels.yaml`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelConfig {
    pub mappings: HashMap<String, String>,
    pub schedule_default: String,
}

/// Representa un guion de video generado por el agente de investigación.
#[derive(Debug, Clone)]
pub struct Script {
    /// El contenido del guion en formato Markdown.
    pub content: String,
    /// El título del video.
    pub title: String,
    /// La descripción del video.
    pub description: String,
    /// Etiquetas sugeridas para el video.
    pub tags: Vec<String>,
}

/// Representa un recurso de video generado y listo para ser publicado.
#[derive(Debug, Clone)]
pub struct VideoAsset {
    /// La ruta local al archivo de video .mp4.
    pub path: PathBuf,
    /// El guion asociado a este video.
    pub script: Script,
    /// El ID del canal de YouTube al que se debe subir.
    pub channel_id: String,
}
