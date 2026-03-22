use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub prompt: String,
    pub duration_seconds: u32,
    pub aspect_ratio: String,
}

pub async fn generate_video(_config: GenerationConfig) -> Result<String, String> {
    // TODO: Implementar llamada real a Vertex AI (Imagen 3 / Veo)
    // Esto requerirá autenticación OAuth2 y llamadas gRPC/REST.
    Err("Not implemented yet".to_string())
}
