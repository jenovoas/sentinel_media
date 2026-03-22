use clap::Parser;
use ignore::WalkBuilder;
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = ".")]
    pub vault: String,

    #[arg(short, long, default_value_t = 0.9)]
    pub min_score: f64,

    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct Candidate {
    pub file: String,
    pub rel_path: String,
    pub score: f64,
    pub status: String,
    pub last_modified_days: u64,
}

#[derive(Serialize, Debug, Default)]
pub struct ScanOutput {
    pub ready: Vec<Candidate>,
    pub pending: Vec<Candidate>,
    pub scores: std::collections::HashMap<String, f64>,
}

fn parse_score(content: &str) -> Option<(f64, String)> {
    let _tag = if content.contains("truthsync:") { "truthsync:" } else if content.contains("scvsync:") { "scvsync:" } else { return None; };
    let re_score = Regex::new(r"score:\s*(\d+\.?\d*)").unwrap();
    let re_status = Regex::new(r"status:\s*(\w+)").unwrap();
    let score = re_score.captures(content).and_then(|cap| cap[1].parse::<f64>().ok()).unwrap_or(0.0);
    let status = re_status.captures(content).map(|cap| cap[1].to_string()).unwrap_or_else(|| "UNKNOWN".to_string());
    Some((score, status))
}

fn scan_file(path: &Path, root: &str, min_score: f64) -> Option<Candidate> {
    if path.extension()?.to_str()? != "md" { return None; }
    let mut file = fs::File::open(path).ok()?;
    let mut buffer = [0; 2048];
    let n = file.read(&mut buffer).ok()?;
    let content = String::from_utf8_lossy(&buffer[..n]);
    if let Some((score, status)) = parse_score(&content) {
        if score >= min_score || status == "UNISON" {
            let abs_path = fs::canonicalize(path).unwrap_or(path.to_path_buf());
            let rel_path = path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string();
            let metadata = fs::metadata(path).ok()?;
            let modified_time = metadata.modified().unwrap_or_else(|_| SystemTime::now());
            let days_since_modified = SystemTime::now()
                .duration_since(modified_time)
                .unwrap_or_default()
                .as_secs() / (60 * 60 * 24);

            return Some(Candidate {
                file: abs_path.to_string_lossy().to_string(),
                rel_path,
                score,
                status,
                last_modified_days: days_since_modified,
            });
        }
    }
    None
}

pub fn run(args: Args) -> anyhow::Result<ScanOutput> {
    let mut walker = WalkBuilder::new(&args.vault);
    walker.hidden(false);
    
    // El soporte para .factoryignore usando walker.add_ignore requiere que el archivo exista.
    // ignore::WalkBuilder ya maneja .ignore por defecto, pero .factoryignore es específico.
    let factory_ignore_path = Path::new(&args.vault).join(".factoryignore");
    if factory_ignore_path.exists() {
        if let Some(err) = walker.add_ignore(factory_ignore_path) {
            if args.verbose {
                eprintln!("⚠️  Error loading .factoryignore: {}", err);
            }
        }
    }

    let entries: Vec<PathBuf> = walker.build()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
        .map(|e| e.path().to_owned())
        .collect();
    let candidates: Vec<Candidate> = entries.par_iter().filter_map(|path| scan_file(path, &args.vault, args.min_score)).collect();
    let output = ScanOutput {
        ready: candidates.clone(),
        pending: vec![],
        scores: candidates.into_iter().map(|c| (c.rel_path.clone(), c.score)).collect(),
    };
    if args.verbose {
        eprintln!("🔥 Sentinel Rust Scanner: Scanned {} files, found {} ready candidates.", entries.len(), output.ready.len());
    }
    Ok(output)
}
