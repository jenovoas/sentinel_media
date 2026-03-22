use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: String,
    pub action: String,
    pub target: String,
    pub status: String,
    pub prev_hash: String,
    pub hash: String,
}

pub struct SecurityAudit {
    log_path: PathBuf,
    last_hash: Arc<Mutex<String>>,
}

impl SecurityAudit {
    pub fn new(log_path: &Path) -> Self {
        // En un sistema real, aquí leeríamos el último hash del archivo para continuar la cadena.
        // Para simplificar, iniciamos con hash cero si está vacío.
        Self {
            log_path: log_path.to_path_buf(),
            last_hash: Arc::new(Mutex::new("00000000000000000000000000000000".to_string())),
        }
    }

    pub fn log(&self, action: &str, target: &str, status: &str) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        let mut last_hash_guard = self.last_hash.lock().unwrap();
        let prev_hash = last_hash_guard.clone();

        // Cadena Criptográfica: Hash(Prev + Datos)
        let payload = format!("{}{}{}{}{}", prev_hash, timestamp, action, target, status);
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let new_hash = format!("{:x}", hasher.finalize());

        let entry = AuditEntry {
            timestamp,
            action: action.to_string(),
            target: target.to_string(),
            status: status.to_string(),
            prev_hash,
            hash: new_hash.clone(),
        };

        // Actualizar estado en memoria
        *last_hash_guard = new_hash;

        // Escribir a disco (JSONL)
        let json_line = serde_json::to_string(&entry)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        writeln!(file, "{}", json_line)?;

        Ok(())
    }
}
