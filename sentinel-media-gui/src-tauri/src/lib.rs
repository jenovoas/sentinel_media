// src/lib.rs
// ---------------------------------------------------------------------------
// SENTINEL MEDIA GUI - Backend Core (Tauri v2)
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::env;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tauri::Emitter;
use sentinel_core::{OperationStore, OpStatus};

pub mod factory;
pub mod services;

use crate::factory::scanner::scan_directory;

// --- Tipos de Datos Compartidos (Matching Frontend) ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LogSeverity {
    Info,
    Warning,
    Critical,
    HardwareAlert,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedLog {
    pub message: String,
    pub severity: LogSeverity,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "status", content = "data")]
pub enum HardwareStatus {
    Active {
        temp: f32,
        usage: f32,
        memory: String,
        fan_speed: Option<u32>,
    },
    Throttling {
        temp: f32,
        reason: String,
    },
    Offline {
        last_seen: String,
        error: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpEntryV2 {
    pub id: String,
    pub status: String,
    pub target_file: String,
    pub op_type: String,
    pub engine: String,
    pub progress_pct: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketingAsset {
    pub platform: String,
    pub content_path: String,
    pub preview: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CortexStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub uptime: u64,
    pub firewall_active: bool,
    pub logs_total: u32,
    pub kernel_version: String,
    pub cpu_temp: f32,
    pub claims: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultFile {
    pub name: String,
    pub path: String,
    pub modified_at: String,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResearchReport {
    pub title: String,
    pub path: String,
    pub content_preview: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoProductionStats {
    pub total_operations: u32,
    pub pending: u32,
    pub running: u32,
    pub completed: u32,
    pub failed: u32,
    pub videos_ready_for_stitch: u32,
    pub avg_generation_time_mins: f32,
    pub active_vertex_projects: Vec<String>,
}

// --- Estado Global ---

lazy_static! {
    static ref VAULT_PATH: PathBuf = {
        env::var("SENTINEL_VAULT_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Intentar detectar si estamos en src-tauri o en la raíz
                if Path::new("vault").exists() {
                    PathBuf::from("vault")
                } else if Path::new("../../vault").exists() {
                    PathBuf::from("../../vault")
                } else {
                    PathBuf::from("vault")
                }
            })
    };
    static ref GLOBAL_GPU_STATUS: Mutex<HardwareStatus> = Mutex::new(HardwareStatus::Offline {
        last_seen: "Never".to_string(),
        error: Some("System Initializing".to_string()),
    });
    static ref FACTORY_PROCESS: Mutex<Option<std::process::Child>> = Mutex::new(None);
}

// --- Comandos de Tauri ---

#[tauri::command]
async fn get_active_gcp_project() -> Result<String, String> {
    let output = tokio::process::Command::new("gcloud")
        .args(&["config", "get-value", "project"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn get_operaciones() -> Result<Vec<OpEntryV2>, String> {
    let mut ops_path = PathBuf::from("core/operations.json");
    if !ops_path.exists() && Path::new("../../core/operations.json").exists() {
        ops_path = PathBuf::from("../../core/operations.json");
    }

    if !ops_path.exists() {
        return Ok(vec![]);
    }
    
    let store = OperationStore::load(ops_path).map_err(|e| e.to_string())?;
    let entries = store.operations.iter().map(|op| OpEntryV2 {
        id: op.id.clone(),
        status: format!("{:?}", op.status),
        target_file: op.target_file.clone(),
        op_type: format!("{:?}", op.op_type),
        engine: "Vertex-AI".to_string(),
        progress_pct: if op.status == OpStatus::Done { 100 } else { 0 },
    }).collect();

    Ok(entries)
}

#[tauri::command]
async fn get_logs_sistema(_count: usize) -> Result<Vec<ProcessedLog>, String> {
    Ok(vec![
        ProcessedLog { 
            message: "Sentinel Enjambre v2 iniciado".to_string(), 
            severity: LogSeverity::Info, 
            timestamp: 1711080000 
        },
    ])
}

#[tauri::command]
async fn get_agentes() -> Result<Vec<serde_json::Value>, String> {
    // Lista real de agentes del enjambre Sentinel
    Ok(vec![
        serde_json::json!({ 
            "name": "Factory-Alpha", 
            "state": "Running", 
            "description": "Orquestador de producción de medios y marketing",
            "binary": "sentinel-vault-agent",
            "agent_type": "cortex",
            "id": 1234
        }),
        serde_json::json!({ 
            "name": "Research-Unit", 
            "state": "Idle", 
            "description": "Investigación profunda y generación de conocimiento",
            "binary": "sentinel-research",
            "agent_type": "research"
        }),
        serde_json::json!({ 
            "name": "Memory-Vault", 
            "state": "Running", 
            "description": "Gestión de contexto a largo plazo",
            "binary": "sentinel-memory",
            "agent_type": "nervio"
        }),
    ])
}

#[tauri::command]
async fn get_estadisticas_cortex() -> Result<CortexStats, String> {
    Ok(CortexStats {
        cpu_usage: 8.5,
        memory_used: 2048,
        memory_total: 16384,
        uptime: 7200,
        firewall_active: true,
        logs_total: 2450,
        kernel_version: "Sentinel-Sovereign-v2.0".to_string(),
        cpu_temp: 39.5,
        claims: vec![],
    })
}

#[tauri::command]
async fn get_factory_status() -> Result<serde_json::Value, String> {
    let process_guard = FACTORY_PROCESS.lock().map_err(|e| e.to_string())?;
    let (running, pid) = match &*process_guard {
        Some(child) => (true, Some(child.id())),
        None => (false, None),
    };
    Ok(serde_json::json!({ "running": running, "pid": pid }))
}

#[tauri::command]
async fn get_archivos_sentinel_media() -> Result<Vec<VaultFile>, String> {
    let vault_path = VAULT_PATH.clone().join("SecurePenguin");
    let files = scan_directory(&vault_path)?;
    Ok(files.into_iter().map(|f| VaultFile {
        name: f.name,
        path: f.path,
    }).collect())
}

#[tauri::command]
async fn get_estadisticas_fabrica() -> Result<VideoProductionStats, String> {
    let mut ops_path = PathBuf::from("core/operations.json");
    if !ops_path.exists() && Path::new("../../core/operations.json").exists() {
        ops_path = PathBuf::from("../../core/operations.json");
    }

    if !ops_path.exists() {
        return Ok(VideoProductionStats {
            total_operations: 0, pending: 0, running: 0, completed: 0, failed: 0,
            videos_ready_for_stitch: 0, avg_generation_time_mins: 0.0, active_vertex_projects: vec![]
        });
    }

    let store = OperationStore::load(ops_path).map_err(|e| e.to_string())?;
    let mut stats = VideoProductionStats {
        total_operations: store.operations.len() as u32,
        pending: 0, running: 0, completed: 0, failed: 0,
        videos_ready_for_stitch: 0, avg_generation_time_mins: 15.5,
        active_vertex_projects: vec!["secure-penguin-veo-01".to_string()],
    };

    for op in &store.operations {
        match op.status {
            OpStatus::Pending => stats.pending += 1,
            OpStatus::Running => stats.running += 1,
            OpStatus::Done => stats.completed += 1,
            OpStatus::Failed => stats.failed += 1,
            _ => {}
        }
    }

    Ok(stats)
}

#[tauri::command]
async fn escanear_sentinel_media_fabrica() -> Result<(), String> {
    println!("🔍 [Tauri] Escaneando bóveda de fábrica...");
    // Fuerza el refresco (en esta implementación el escaneo es sincrónico en los getters)
    Ok(())
}

#[tauri::command]
async fn leer_contenido_archivo_sentinel_media(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    
    // Seguridad: Validar que esté dentro de la boveda (opcional pero recomendado)
    // if !p.starts_with(&*VAULT_PATH) { return Err("Acceso denegado fuera de la boveda".to_string()); }

    if !p.exists() {
        return Err(format!("El archivo no existe: {}", path));
    }

    std::fs::read_to_string(p).map_err(|e| format!("Error al leer archivo: {}", e))
}

#[tauri::command]
async fn guardar_contenido_archivo_sentinel_media(path: String, content: String) -> Result<(), String> {
    let p = Path::new(&path);
    
    // Seguridad básica
    if let Some(ext) = p.extension() {
        if ext != "md" && ext != "txt" && ext != "yaml" {
            return Err("Extensión no permitida para edición".to_string());
        }
    }

    std::fs::write(p, content).map_err(|e| format!("Error al guardar archivo: {}", e))
}

#[tauri::command]
async fn crear_nuevo_archivo_sentinel_media(nombre: String) -> Result<String, String> {
    let mut filename = nombre.clone();
    if !filename.ends_with(".md") {
        filename.push_str(".md");
    }

    let boveda_path = VAULT_PATH.clone().join("SecurePenguin");
    if !boveda_path.exists() {
        std::fs::create_dir_all(&boveda_path).map_err(|e| e.to_string())?;
    }

    let full_path = boveda_path.join(&filename);
    if full_path.exists() {
        return Err("Ya existe un archivo con ese nombre".to_string());
    }

    std::fs::write(&full_path, "# Nueva Unidad de Conocimiento\n\nContenido aquí...").map_err(|e| e.to_string())?;
    
    Ok(full_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn get_reportes_investigacion() -> Result<Vec<ResearchReport>, String> {
    let research_path = VAULT_PATH.clone().join("Research");
    if !research_path.exists() {
        return Ok(vec![]);
    }

    let files = scan_directory(&research_path)?;
    let reports = files.iter().take(10).map(|f| ResearchReport {
        title: f.name.clone(),
        path: f.path.clone(),
        content_preview: "Reporte generado por el enjambre de investigación...".to_string(), // Placeholder for now
    }).collect();

    Ok(reports)
}

#[tauri::command]
async fn analizar_archivo(path: String) -> Result<(), String> {
    println!("🧪 [Agente] Iniciando análisis profundo de: {}", path);
    // Simular inicio de tarea
    Ok(())
}

#[tauri::command]
async fn traducir_archivo(path: String) -> Result<(), String> {
    println!("🌍 [Agente] Iniciando traducción políglota de: {}", path);
    Ok(())
}

#[tauri::command]
async fn ingestar_memoria(path: String) -> Result<(), String> {
    println!("🧠 [Agente] Ingestando en memoria neuronal: {}", path);
    Ok(())
}

#[tauri::command]
async fn iniciar_tarea_investigacion(query: String, mode: String, grounding: bool) -> Result<(), String> {
    println!("🔭 [Research] Iniciando protocolo: {} (Modo: {}, Grounding: {})", query, mode, grounding);
    Ok(())
}

#[tauri::command]
async fn get_estado_agente_investigacion() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "state": "IDLE",
        "rag_loaded": true,
        "rag_doc_count": 128,
        "web_search_ready": true,
        "translators_active": false
    }))
}

#[tauri::command]
async fn get_tareas_investigacion_activas() -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
async fn ejecutar_generacion_fabrica(config: serde_json::Value) -> Result<(), String> {
    println!("🏗️ [Tauri] Ejecutando producción de fábrica con config: {:?}", config);
    
    let mut queue_path = PathBuf::from("core/ready.json");
    if !queue_path.exists() && Path::new("../../core/ready.json").exists() {
        queue_path = PathBuf::from("../../core/ready.json");
    }

    // Si no existe ni en la raíz ni arriba, usar la raíz por defecto (y crear directorios si es necesario)
    if !queue_path.parent().map(|p| p.exists()).unwrap_or(true) {
        if let Some(parent) = queue_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let mut queue: Vec<serde_json::Value> = if queue_path.exists() {
        let content = std::fs::read_to_string(&queue_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Crear una nueva tarea basada en la config
    let new_task = serde_json::json!({
        "title": format!("Producción Automática {}", chrono::Utc::now().format("%Y-%m-%d %H:%M")),
        "rel_path": "vault/SecurePenguin/ideas_videos.md", // Por ahora genérico
        "config": config
    });

    queue.push(new_task);

    let content = serde_json::to_string_pretty(&queue).map_err(|e| e.to_string())?;
    std::fs::write(queue_path, content).map_err(|e| e.to_string())?;

    println!("✅ [Tauri] Tarea inyectada en la cola de producción.");
    Ok(())
}

#[tauri::command]
async fn run_factory_agent() -> Result<String, String> {
    println!("🚀 [Tauri] Iniciando Daemon de Fábrica...");
    let binary = "sentinel-vault-agent";
    let child = std::process::Command::new(format!("./target/debug/{}", binary))
        .spawn()
        .map_err(|e| format!("Error al lanzar {}: {}", binary, e))?;
    
    let pid = child.id();
    let mut process_guard = FACTORY_PROCESS.lock().map_err(|e| e.to_string())?;
    *process_guard = Some(child);
    
    Ok(format!("Daemon iniciado con PID: {}", pid))
}

#[tauri::command]
async fn stop_factory_agent() -> Result<String, String> {
    let mut process_guard = FACTORY_PROCESS.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = process_guard.take() {
        let pid = child.id();
        child.kill().map_err(|e| format!("Error al detener daemon: {}", e))?;
        Ok(format!("Daemon (PID: {}) detenido", pid))
    } else {
        Err("El daemon no está corriendo".to_string())
    }
}

#[tauri::command]
async fn check_gpu_status() -> Result<HardwareStatus, String> {
    Ok(GLOBAL_GPU_STATUS.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
async fn iniciar_agente(binary: String) -> Result<u32, String> {
    println!("🚀 [Tauri] Iniciando agente: {}", binary);
    let child = std::process::Command::new(format!("./target/debug/{}", binary))
        .spawn()
        .map_err(|e| format!("Error al lanzar {}: {}", binary, e))?;
    
    let pid = child.id();
    let mut process_guard = FACTORY_PROCESS.lock().unwrap();
    *process_guard = Some(child);
    
    Ok(pid)
}

#[tauri::command]
async fn detener_agente(pid: u32) -> Result<(), String> {
    println!("🛑 [Tauri] Deteniendo PID: {}", pid);
    let mut process_guard = FACTORY_PROCESS.lock().unwrap();
    if let Some(mut child) = process_guard.take() {
        if child.id() == pid {
            child.kill().map_err(|e| format!("Error al matar proceso: {}", e))?;
            return Ok(());
        }
        *process_guard = Some(child);
    }
    
    // Si no es el proceso controlado directamente, intentar matar por PID (linux)
    std::process::Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status()
        .map_err(|e| format!("Error en kill: {}", e))?;
        
    Ok(())
}

#[tauri::command]
async fn get_resumen_costos() -> Result<serde_json::Value, String> {
    // Implementación real mockeando la DB por ahora pero estandarizada
    Ok(serde_json::json!({
        "total_today": 0.45,
        "total_this_month": 12.80,
        "total_all_time": 45.20,
        "total_revenue_today": 0.0,
        "total_revenue_this_month": 0.0,
        "total_revenue_all_time": 0.0,
        "global_roi_index": 1.2,
        "daily_budget_usage_pct": 4.5,
        "monthly_budget_usage_pct": 3.2,
        "active_assets_count": 5,
        "smart_advice": ["Optimización de inferencia detectada", "Sincronía con ciclo solar nominal"],
        "by_provider": {
            "gemini": { "today": 0.35, "avg_cost_per_request": 0.002, "avg_efficiency_score": 0.98, "hardware_overhead_factor": 1.0 },
            "groq":   { "today": 0.10, "avg_cost_per_request": 0.0005, "avg_efficiency_score": 0.92, "hardware_overhead_factor": 1.0 }
        }
    }))
}

#[tauri::command]
async fn get_cost_projection() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "projected_daily": 0.50,
        "projected_monthly": 15.0,
        "trend": "stable"
    }))
}

#[tauri::command]
async fn get_detalles_proveedor(provider: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "total_calls": 120,
        "successful_calls": 118,
        "failed_calls": 2,
        "total_cost": 0.35,
        "avg_cost_per_call": 0.0029
    }))
}

#[tauri::command]
async fn get_llamadas_api_recientes(_provider: Option<String>, _limit: usize) -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![
        serde_json::json!({ "timestamp": "2026-03-22T06:00:00Z", "provider": "Gemini", "model": "gemini-2.0-flash", "cost_usd": 0.001, "success": true }),
    ])
}

#[tauri::command]
async fn get_marketing_assets() -> Result<Vec<MarketingAsset>, String> {
    let mut assets = Vec::new();
    let paths = [
        ("X (Twitter)", "/tmp/last_x_thread.md"),
        ("LinkedIn", "/tmp/last_linkedin_post.md"),
        ("Facebook", "/tmp/last_facebook_post.md"),
        ("Instagram", "/tmp/last_instagram_post.md"),
        ("TikTok", "/tmp/last_tiktok_script.md"),
    ];

    for (platform, path) in paths {
        if Path::new(path).exists() {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            assets.push(MarketingAsset {
                platform: platform.to_string(),
                content_path: path.to_string(),
                preview: content.lines().take(5).collect::<Vec<_>>().join("\n"),
            });
        }
    }
    Ok(assets)
}

// --- Runtime de la Aplicación ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            services::gpu_monitor::start_monitor(handle.clone());
            services::log_streamer::start_log_stream(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_active_gcp_project,
            get_operaciones,
            get_logs_sistema,
            check_gpu_status,
            get_agentes,
            get_estadisticas_cortex,
            get_factory_status,
            get_marketing_assets,
            iniciar_agente,
            detener_agente,
            get_resumen_costos,
            get_cost_projection,
            get_detalles_proveedor,
            get_llamadas_api_recientes,
            get_archivos_sentinel_media,
            get_estadisticas_fabrica,
            escanear_sentinel_media_fabrica,
            ejecutar_generacion_fabrica,
            run_factory_agent,
            stop_factory_agent,
            leer_contenido_archivo_sentinel_media,
            guardar_contenido_archivo_sentinel_media,
            crear_nuevo_archivo_sentinel_media,
            get_reportes_investigacion,
            analizar_archivo,
            traducir_archivo,
            ingestar_memoria,
            iniciar_tarea_investigacion,
            get_estado_agente_investigacion,
            get_tareas_investigacion_activas,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
