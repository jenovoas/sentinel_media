use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Deserializa una clave API que puede ser un string o un array de strings.
/// Si es un array, las claves se unen en un solo string separado por comas.
fn deserialize_api_keys<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let v: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Array(arr) => {
            let keys: Vec<String> = arr
                .into_iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect();
            if keys.is_empty() {
                Ok(None)
            } else {
                Ok(Some(keys.join(",")))
            }
        }
        _ => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FactoryConfig {
    #[serde(default, deserialize_with = "deserialize_api_keys")]
    pub gemini_api_keys: Option<String>,
    #[serde(default)]
    pub gcloud_project_id: Option<String>,
    #[serde(default)]
    pub gcloud_region: Option<String>,
    
    // Nuevos campos GCS
    #[serde(default)]
    pub gcs_video_bucket: Option<String>,
    #[serde(default)]
    pub gcs_research_bucket: Option<String>,
    #[serde(default)]
    pub gcp_service_account_path: Option<String>,

    // Otras claves de API
    #[serde(default)]
    pub groq_api_key: Option<String>,
    #[serde(default)]
    pub openai_api_key: Option<String>,
    #[serde(default)]
    pub openai_session_token: Option<String>,
    #[serde(default)]
    pub antigravity_session_token: Option<String>,
    #[serde(default)]
    pub perplexity_api_key: Option<String>,
}

impl FactoryConfig {
    /// Carga la configuración desde múltiples fuentes con un orden de prioridad.
    ///
    /// Prioridad:
    /// 1. Variable de entorno `FACTORY_CONFIG_PATH`.
    /// 2. Archivo de configuración en el directorio de config del sistema (`~/.config/sentinel_media/config.json`).
    /// 3. Archivo `sentinel_keys.json` en el directorio de trabajo actual.
    /// 4. Variables de entorno individuales para cada clave (ej. `GEMINI_API_KEYS`).
    pub fn load() -> Result<Self> {
        let from_file = Self::load_from_file().unwrap_or_default();
        let from_env = Self::load_from_env();

        // Fusionar: las variables de entorno tienen prioridad
        Ok(Self {
            gemini_api_keys: from_env.gemini_api_keys.or(from_file.gemini_api_keys),
            gcloud_project_id: from_env.gcloud_project_id.or(from_file.gcloud_project_id),
            gcloud_region: from_env.gcloud_region.or(from_file.gcloud_region),
            gcs_video_bucket: from_env.gcs_video_bucket.or(from_file.gcs_video_bucket),
            gcs_research_bucket: from_env.gcs_research_bucket.or(from_file.gcs_research_bucket),
            gcp_service_account_path: from_env.gcp_service_account_path.or(from_file.gcp_service_account_path),
            groq_api_key: from_env.groq_api_key.or(from_file.groq_api_key),
            openai_api_key: from_env.openai_api_key.or(from_file.openai_api_key),
            openai_session_token: from_env.openai_session_token.or(from_file.openai_session_token),
            antigravity_session_token: from_env.antigravity_session_token.or(from_file.antigravity_session_token),
            perplexity_api_key: from_env.perplexity_api_key.or(from_file.perplexity_api_key),
        })
    }

    /// Carga la configuración desde un archivo.
    fn load_from_file() -> Option<Self> {
        let config_path = 
            // 1. Variable de entorno `FACTORY_CONFIG_PATH`
            std::env::var("FACTORY_CONFIG_PATH").map(PathBuf::from)
            // 2. Directorio de config del sistema
            .or_else(|_| {
                dirs::config_dir()
                    .map(|p| p.join("sentinel_media/config.json"))
                    .ok_or(())
            })
            // 3. Directorio de trabajo actual
            .or_else(|_| std::env::current_dir().map(|p| p.join("sentinel_keys.json")).map_err(|_| ()));

        if let Ok(path) = config_path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    return serde_json::from_str(&content).ok();
                }
            }
        }
        None
    }
    
    /// Carga la configuración desde variables de entorno.
    fn load_from_env() -> Self {
        dotenvy::dotenv().ok(); // Carga .env si existe

        Self {
            gemini_api_keys: std::env::var("GEMINI_API_KEYS")
                .or_else(|_| std::env::var("GOOGLE_AI_API_KEY"))
                .or_else(|_| std::env::var("GOOGLE_API_KEY")).ok(),
            gcloud_project_id: std::env::var("GCLOUD_PROJECT_ID").ok(),
            gcloud_region: std::env::var("GCLOUD_REGION").ok(),
            gcs_video_bucket: std::env::var("GCS_VIDEO_BUCKET").ok(),
            gcs_research_bucket: std::env::var("GCS_RESEARCH_BUCKET").ok(),
            gcp_service_account_path: std::env::var("GCP_SERVICE_ACCOUNT_PATH").ok(),
            groq_api_key: std::env::var("GROQ_API_KEY").ok(),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_session_token: std::env::var("OPENAI_SESSION_TOKEN").ok(),
            antigravity_session_token: std::env::var("ANTIGRAVITY_SESSION_TOKEN").ok(),
            perplexity_api_key: std::env::var("PERPLEXITY_API_KEY").ok(),
        }
    }
}
