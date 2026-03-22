use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::path::PathBuf;
use serde_json::json;
use sentinel_core::{load_agent_skill, FactoryConfig};

pub mod telos_indexer;
pub mod rate_limiter;
pub mod http_auth;
pub mod hooks;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub file: Option<String>,
    #[arg(short, long)]
    pub prompt: Option<String>,
    #[arg(long)]
    pub imagina: bool,
    #[arg(long)]
    pub intuicion: bool,
    #[arg(long)]
    pub deep: bool,
    #[arg(long)]
    pub refactor: bool,
    #[arg(long)]
    pub translate: bool,
    #[arg(long, default_value = "es")]
    pub target_lang: String,
    #[arg(long)]
    pub interactive: bool,
    #[arg(long)]
    pub system: bool,
    #[arg(long, default_value = "warm")]
    pub memory_tier: String,
    #[arg(long)]
    pub telos_context: bool,
    #[arg(long)]
    pub groq: bool,
    #[arg(long)]
    pub openai: bool,
    #[arg(long)]
    pub antigravity: bool,
    #[arg(long)]
    pub perplexity: bool,
    #[arg(long)]
    pub hook: Vec<String>,
}

// ... (other structs remain private to the library)

#[derive(Serialize, Deserialize, Debug)]
pub struct VertexRequest {
    pub contents: Vec<VertexContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<VertexContent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VertexContent {
    pub parts: Vec<VertexPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VertexPart {
    pub text: String,
}

#[derive(Deserialize, Debug)]
struct VertexResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: VertexContentResponse,
}

#[derive(Deserialize, Debug)]
struct VertexContentResponse {
    parts: Vec<VertexPart>,
}

pub struct SentinelResearch {
    client: Client,
}

impl SentinelResearch {
    pub fn new() -> Result<Self> {
        Ok(Self { client: Client::new() })
    }

    pub async fn synthesize_vertex(&self, config: &FactoryConfig, system_msg: &str, user_msg: &str) -> Result<String> {
        let project = config.gcloud_project_id.as_ref()
            .context("Error: gcloud_project_id no configurado en FactoryConfig")?;
        let region = config.gcloud_region.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("us-central1");
        
        let model = "gemini-2.0-flash-001";

        // Obtener Token via gcloud CLI (Simplificado para esta fase)
        let token_out = Command::new("gcloud").args(&["auth", "print-access-token"]).output()
            .context("Error al ejecutar gcloud auth print-access-token. ¿Está instalado y autenticado?")?;
        
        if !token_out.status.success() {
            anyhow::bail!("Error al obtener token de gcloud: {}", String::from_utf8_lossy(&token_out.stderr));
        }
        
        let token = String::from_utf8_lossy(&token_out.stdout).trim().to_string();
        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent", 
            region, project, region, model
        );
        
        let body = json!({
            "contents": [{ "role": "user", "parts": [{ "text": user_msg }] }],
            "systemInstruction": { "parts": [{ "text": system_msg }] }
        });

        let res = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await?;
            
        if res.status().is_success() {
            let json: VertexResponse = res.json().await?;
            if let Some(candidate) = json.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    return Ok(part.text.clone());
                }
            }
            anyhow::bail!("Vertex AI retornó una respuesta vacía o malformada");
        } else {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            anyhow::bail!("Vertex AI Error ({}): {}", status, err_text);
        }
    }
}

pub async fn run(args: Args) -> Result<String> {
    let config = FactoryConfig::load()?;
    let research = SentinelResearch::new()?;
    
    let mut input_content = String::new();
    if let Some(ref file_path) = args.file {
        input_content = fs::read_to_string(file_path)
            .with_context(|| format!("No se pudo leer el archivo: {}", file_path))?;
    }

    let system_msg = "Eres Sentinel Media - Assistant. Genera un guion para un video de YouTube basado en el contenido proporcionado. El guion debe ser profesional, atractivo y seguir una estructura clara.";
    let user_msg = if let Some(ref p) = args.prompt {
        format!("INSTRUCCIÓN: {}\n\nCONTENIDO:\n{}", p, input_content)
    } else {
        input_content
    };

    println!("🚀 Generando investigación con Gemini 2.0 Flash...");
    research.synthesize_vertex(&config, &system_msg, &user_msg).await
}
