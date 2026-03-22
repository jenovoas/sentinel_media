use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use google_youtube3::{api::{Video, VideoSnippet, VideoStatus}, YouTube};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use yup_oauth2::{ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Sentinel Publisher: YouTube Upload Engine")]
pub struct Args {
    #[arg(long)]
    pub file: String,
    #[arg(long)]
    pub channel: String,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long, default_value = "private")]
    pub privacy: String,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub channel_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub token_cache_path: String,
    pub default_category: u32,
    pub default_tags: Vec<String>,
    pub default_playlist: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelsConfig {
    pub channels: HashMap<String, Channel>,
}

pub async fn run(args: Args) -> Result<()> {
    let config_content = fs::read_to_string("channels.yaml")
        .context("No se pudo leer channels.yaml en la raíz del proyecto")?;
    
    // Expandir variables de entorno en el string YAML antes de parsear
    let expanded_config = shellexpand::env(&config_content)
        .map_err(|e| anyhow::anyhow!("Error expandiendo variables en channels.yaml: {}", e))?
        .to_string();

    let channels_config: ChannelsConfig = serde_yaml::from_str(&expanded_config)
        .context("Error al parsear channels.yaml (después de expansión)")?;
    
    let channel_config = channels_config.channels.get(&args.channel).cloned()
        .ok_or_else(|| anyhow::anyhow!("Canal '{}' no encontrado en channels.yaml. Disponibles: {:?}", 
            args.channel, channels_config.channels.keys().collect::<Vec<_>>()))?;
    
    upload_video(args, channel_config).await
}

async fn upload_video(args: Args, channel_config: Channel) -> Result<()> {
    println!("{}", "🔱 Sentinel Publisher: Initiating Upload Sequence...".cyan().bold());
    println!("🔑 Authenticating with YouTube API for channel: {}", args.channel.yellow());
    
    let secret = ApplicationSecret {
        client_id: channel_config.client_id.clone(),
        client_secret: channel_config.client_secret.clone(),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
        redirect_uris: vec!["http://127.0.0.1:8080".to_string()],
        ..Default::default()
    };

    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();
    let client = hyper::Client::builder().build(https);

    // Resolver path de cache de token (soporta ~)
    let token_path_raw = shellexpand::tilde(&channel_config.token_cache_path).to_string();
    let token_path = PathBuf::from(&token_path_raw);
    
    // Asegurar que el directorio de cache existe
    if let Some(parent) = token_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk(&token_path_raw)
        .hyper_client(client.clone())
        .build()
        .await?;

    let hub = YouTube::new(client, auth);

    let video_path = PathBuf::from(&args.file);
    if !video_path.exists() {
        return Err(anyhow::anyhow!("❌ Archivo de video no encontrado: {}", args.file));
    }

    let title = args.title.unwrap_or_else(|| 
        video_path.file_stem().unwrap_or_default().to_string_lossy().to_string()
    );
    let description = args.description.unwrap_or_else(|| 
        "Uploaded via Sentinel Media Publisher (Rust Native)".to_string()
    );
    let privacy_status = match args.privacy.as_str() {
        "public" => "public",
        "unlisted" => "unlisted",
        _ => "private",
    };

    println!("📄 Metadata Preparada:\n   Título: {}\n   Privacidad: {}", title.yellow(), privacy_status.yellow());

    if args.dry_run {
        println!("{}", "⚠️ MODO DRY RUN: Saltando subida real.".yellow());
        return Ok(());
    }

    let mut req = Video::default();
    req.snippet = Some(VideoSnippet {
        title: Some(title),
        description: Some(description),
        tags: Some(channel_config.default_tags.clone()),
        category_id: Some(channel_config.default_category.to_string()),
        ..Default::default()
    });
    req.status = Some(VideoStatus {
        privacy_status: Some(privacy_status.to_string()),
        ..Default::default()
    });

    println!("🚀 Subiendo contenido (esto puede tardar varios minutos)...");
    
    let video_file = fs::File::open(&video_path)?;
    
    // Usar upload_resumable para videos grandes (>5MB)
    let (response, video) = hub.videos().insert(req)
        .upload_resumable(&video_file, "video/mp4".parse().unwrap())
        .await?;

    if response.status().is_success() {
        println!("{}", "✅ ¡Subida Exitosa!".green().bold());
        if let Some(id) = video.id {
            println!("   URL del Video: https://youtu.be/{}", id);
        }
    } else {
        println!("{}", "❌ Error en la subida.".red().bold());
        println!("   Status: {}", response.status());
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
        let err_body = String::from_utf8_lossy(&body_bytes);
        println!("   Error: {}", err_body);
        anyhow::bail!("YouTube Insert Error: {}", err_body);
    }

    Ok(())
}
