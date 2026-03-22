/*
use sentinel_media_core::{OpStatus, OperationStore};
use me60os_core::agent_manager::AgentSPA;
use me60os_core::cortex::CortexEngine;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};

pub struct CloudOpsAgent {
    name: String,
    store_path: String,
    last_check: SystemTime,
}

impl CloudOpsAgent {
    pub fn new(name: &str, store_path: &str) -> Self {
        Self {
            name: name.to_string(),
            store_path: store_path.to_string(),
            last_check: SystemTime::UNIX_EPOCH,
        }
    }

    /// Consulta a Vertex AI si la operación ha terminado
    fn check_cloud_status(&self, op_id: &str) -> Option<bool> {
        // gcloud ai operations describe ID --format="value(done)"
        let output = Command::new("gcloud")
            .args(&[
                "ai",
                "operations",
                "describe",
                op_id,
                "--format=value(done)",
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            return Some(result == "true");
        }
        None
    }

    /// Descarga el video desde GCS al sistema local
    fn download_video(&self, gcs_uri: &str, local_path: &str) -> bool {
        println!(
            "📥 [CloudOps] Detectado video finalizado. Descargando: {} -> {}",
            gcs_uri, local_path
        );

        // Aseguramos que el directorio destino existe
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

impl AgentSPA for CloudOpsAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn percibir(&mut self, _cortex: &CortexEngine) -> bool {
        let now = SystemTime::now();
        // Evitamos saturar la API de Google: revisamos cada 30 segundos
        if now
            .duration_since(self.last_check)
            .unwrap_or(Duration::from_secs(0))
            < Duration::from_secs(30)
        {
            return true;
        }
        self.last_check = now;

        let path = Path::new(&self.store_path);
        let mut store = match OperationStore::load(path) {
            Ok(s) => s,
            Err(_) => return true,
        };

        let mut changed = false;
        let mut ops_to_check = Vec::new();

        // Recolectamos IDs de operaciones en vuelo
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
                            println!(
                                "✅ [CloudOps] Operación finalizada y archivo descargado: {}",
                                op.target_file
                            );
                        } else {
                            println!("⚠️ [CloudOps] Error al descargar de GCS: {}", uri);
                        }
                    }
                }
            }
        }

        if changed {
            let _ = store.save(path);
        }

        true
    }

    fn decidir(&mut self) -> String {
        "NO_OP".to_string()
    }

    fn actuar(&mut self, _action: String) {}
}
*/
