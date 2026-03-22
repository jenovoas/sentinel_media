use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use std::path::Path;
use std::process::Command;
use sentinel_media_core::FactoryConfig;

pub async fn get_gcloud_token() -> Result<String> {
    let output = Command::new("gcloud")
        .args(&["auth", "print-access-token"])
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        anyhow::bail!("gcloud auth failed")
    }
}

pub async fn generate_video_vertex(
    client: &Client,
    project: &str,
    location: &str,
    prompt: &str,
    output: &str,
    duration: u32,
    aspect: &str,
    config: &FactoryConfig,
) -> Result<()> {
    let token = get_gcloud_token().await?;
    let url = format!(
        "https://{}-aiplatform.googleapis.com/v1beta1/projects/{}/locations/{}/publishers/google/models/veo-3.0-fast-generate-001:predictLongRunning",
        location, project, location
    );

    let bucket = config.gcs_video_bucket.as_deref().unwrap_or("sentinel_media-video-output");

    let payload = json!({
        "instances": [{
            "prompt": prompt
        }],
        "parameters": {
            "aspectRatio": aspect,
            "durationSeconds": duration,
            "sampleCount": 1,
            "storageUri": format!("gs://{}/{}", bucket, output)
        }
    });

    println!("🎬 Generating Video via Veo 3 Fast (Vertex AI)...");
    println!("   Prompt: {}", prompt);
    println!("   Duration: {}s | Aspect: {}", duration, aspect);

    for attempt in 1..=3 {
        let res = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => {
                let body: serde_json::Value = r.json().await?;
                if let Some(op_name) = body["name"].as_str() {
                    println!("📤 Video generation started. Operation: {}", op_name);
                    println!("   Use 'gcloud ai operations describe {} --region={}' to check status.", op_name, location);
                    
                    let store_path = Path::new(".sentinel/operations.json");
                    let mut store = sentinel_media_core::OperationStore::load(store_path).unwrap_or_default();
                    
                    store.add(sentinel_media_core::Operation {
                        id: op_name.to_string(),
                        op_type: sentinel_media_core::OpType::VideoGeneration,
                        status: sentinel_media_core::OpStatus::Running,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        prompt: prompt.to_string(),
                        target_file: output.to_string(),
                        gcs_uri: Some(format!("gs://{}/{}", bucket, output)),
                        metadata: None,
                    });
                    
                    store.save(store_path).expect("Failed to save operation state");
                    println!("💾 Operation state saved to .sentinel/operations.json");
                    
                    return Ok(());
                }
                anyhow::bail!("Veo response missing operation name: {:?}", body);
            },
            Ok(r) => {
                 return anyhow::bail!("Veo 3 Video Generation failed: {:?}", r.text().await?)
            },
            Err(e) => {
                eprintln!("Attempt {} failed: {}", attempt, e);
                if attempt == 3 {
                    return Err(e.into());
                }
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }
        }
    }
    unreachable!();
}
