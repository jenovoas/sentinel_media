// src/cloud_ops_agent.rs
//! ☁️ Sentinel Vault: CLOUD OPS AGENT ☁️
//! ---------------------------------------------------------------------------
//! Monitorea el estado de operaciones en Google Cloud y gestiona descargas.

use anyhow::Result;
use colored::Colorize;
use sentinel_core::{OperationStore, OpStatus};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};

pub struct CloudOpsAgent {
    pub name: String,
    pub store_path: String,
    pub last_check: SystemTime,
}

impl CloudOpsAgent {
    pub fn new(name: &str, store_path: &str) -> Self {
        Self {
            name: name.to_string(),
            store_path: store_path.to_string(),
            last_check: SystemTime::UNIX_EPOCH,
        }
    }

    /// Revisa el estado de las operaciones en la nube
    pub async fn tick(&mut self) -> Result<()> {
        let now = SystemTime::now();
        if now.duration_since(self.last_check).unwrap_or(Duration::from_secs(0)) < Duration::from_secs(30) {
            return Ok(());
        }
        self.last_check = now;

        let path = Path::new(&self.store_path);
        let mut store = match OperationStore::load(path) {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        let mut changed = false;
        let mut ops_to_check = Vec::new();

        for op in &store.operations {
            if op.status == OpStatus::Running {
                ops_to_check.push(op.clone());
            }
        }

        for op in ops_to_check {
            if let Some(done) = self.check_cloud_status(&op.id) {
                if done {
                    if let Some(uri) = &op.gcs_uri {
                        if self.download_video(uri, &op.target_file) {
                            store.mark_done(&op.id);
                            changed = true;
                            println!("✅ [{}] Operación finalizada y archivo descargado: {}", 
                                self.name.blue().bold(), op.target_file.yellow());
                        }
                    }
                }
            }
        }

        if changed {
            let _ = store.save(path);
        }

        Ok(())
    }

    fn check_cloud_status(&self, op_id: &str) -> Option<bool> {
        let output = Command::new("gcloud")
            .args(&["ai", "operations", "describe", op_id, "--format=value(done)"])
            .output()
            .ok()?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
            return Some(result == "true");
        }
        None
    }

    fn download_video(&self, gcs_uri: &str, local_path: &str) -> bool {
        println!("📥 [{}] Descargando activo: {} -> {}", self.name.blue(), gcs_uri, local_path);
        if let Some(parent) = Path::new(local_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let status = Command::new("gcloud")
            .args(&["storage", "cp", gcs_uri, local_path])
            .status()
            .ok();
        status.map_or(false, |s| s.success())
    }
}
