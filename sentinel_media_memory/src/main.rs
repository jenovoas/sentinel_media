use anyhow::Result;
use sentinel_media_memory::{CandleEmbedder, Document, VectorStore};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "sentinel_media_memory")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Ingest {
        #[arg(short, long)]
        path: String,
    },
    Query {
        text: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = PathBuf::from(format!("{}/.sentinel_memory.json", home));

    match &cli.command {
        Commands::Ingest { path } => {
            println!("🚀 Initializing Candle (Pure Rust) Brain...");
            let mut embedder = CandleEmbedder::new()?;
            let mut store = VectorStore::load(&db_path)?;

            let mut count = 0;
            let walker = WalkDir::new(path).into_iter().filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target" && name != "node_modules"
            });

            for entry in walker.filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let ext = entry.path().extension().and_then(|e| e.to_str());
                    if !matches!(ext, Some("md") | Some("json") | Some("txt")) {
                        continue;
                    }

                    let file_path = entry.path().to_str().unwrap();
                    if file_path.contains("/_Agentes/") || file_path.contains("/.git/") {
                        continue;
                    }
                    let content = match fs::read_to_string(file_path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    if content.trim().is_empty() || content.len() > 100_000 {
                        continue;
                    }

                    println!("🧠 Learning: {}", file_path);

                    let truncated = if content.len() > 5000 {
                        let mut end = 5000;
                        while !content.is_char_boundary(end) && end > 0 {
                            end -= 1;
                        }
                        &content[..end]
                    } else {
                        &content
                    };
                    match embedder.embed(truncated) {
                        Ok(vector) => {
                            store.add(Document {
                                path: file_path.to_string(),
                                content: content.clone(),
                                vector,
                            });
                            count += 1;
                        }
                        Err(e) => eprintln!("   ⚠️ Error embedding: {}", e),
                    }
                }
            }

            store.save(&db_path)?;
            println!("\n✅ Ingested {} documents to {:?}", count, db_path);
            println!("📊 Total in memory: {} documents", store.documents.len());
        }
        Commands::Query { text, limit } => {
            println!("🔍 Neural Query: '{}'", text);

            let mut embedder = CandleEmbedder::new()?;
            let store = VectorStore::load(&db_path)?;

            if store.documents.is_empty() {
                println!("⚠️ No documents in memory. Run 'ingest' first.");
                return Ok(());
            }

            let query_vec = embedder.embed(text)?;

            let results = store.search(&query_vec, *limit);

            println!("\n📚 Top {} results:", results.len());
            for (i, (doc, score)) in results.iter().enumerate() {
                let preview: String = doc.content.chars().take(150).collect();
                println!("\n{}. 📄 {} (score: {:.3})", i + 1, doc.path, score);
                println!("   {}...", preview.replace('\n', " "));
            }
        }
    }

    Ok(())
}
