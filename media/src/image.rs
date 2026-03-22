use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use base64::{engine::general_purpose, Engine as _};

pub async fn generate_image_vertex(client: &Client, project: &str, location: &str, prompt: &str, output: &str, aspect: &str) -> Result<()> {
    let token = crate::video::get_gcloud_token().await?;
    let url = format!("https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/imagen-3.0-generate-001:predict", location, project, location);
    
    let payload = json!({
        "instances": [{
            "prompt": prompt
        }],
        "parameters": {
            "sampleCount": 1,
            "aspectRatio": aspect,
            "personGeneration": "allow_adult"
        }
    });

    println!("🎨 Generating Image via Vertex AI...");
    let res = client.post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await?;
        if let Some(b64) = body["predictions"][0]["bytesBase64Encoded"].as_str() {
            let bytes = general_purpose::STANDARD.decode(b64)?;
            std::fs::write(output, bytes)?;
            println!("✅ Image saved: {}", output);
            return Ok(());
        }
        anyhow::bail!("Vertex response missing image data");
    }
    
    anyhow::bail!("Vertex Image Generation failed: {:?}", res.text().await?)
}
