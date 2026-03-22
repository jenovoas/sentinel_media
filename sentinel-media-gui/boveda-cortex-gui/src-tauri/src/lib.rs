mod s60_math;
pub use s60_math::Sexagesimal;

use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tauri::Emitter;
use me60os::crystal_store::CrystalStore;
use me60os::spa::SPA;

pub mod factory;
pub mod services;
pub mod redis_sync;

// === Senior Engineering Patterns: Robust Backend ===

/// Enum robusto para estados de hardware con datos asociados.
/// Tag 'status' permite discriminación directa en TypeScript.
#[derive(Serialize, Deserialize, Debug, Clone)]
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

/// Trait para estandarizar comportamiento de visualización en el Dashboard.
pub trait DashboardMetric {
    /// Determina el nivel de severidad para UI (normal/warning/critical)
    fn get_severity(&self) -> String;

    /// Serializa la métrica para consumo del frontend
    fn to_frontend_payload(&self) -> serde_json::Value;
}

impl DashboardMetric for HardwareStatus {
    fn get_severity(&self) -> String {
        match self {
            HardwareStatus::Active { temp, usage, .. } => {
                if *temp > 85.0 || *usage > 95.0 {
                    "critical".to_string()
                } else if *temp > 75.0 || *usage > 85.0 {
                    "warning".to_string()
                } else {
                    "normal".to_string()
                }
            },
            HardwareStatus::Throttling { .. } => "warning".to_string(),
            HardwareStatus::Offline { .. } => "critical".to_string(),
        }
    }

    fn to_frontend_payload(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

// Contador global Round-Robin para balanceo de llaves API
static API_KEY_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Estado global del proceso del agente de fábrica
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref FACTORY_PROCESS: Mutex<Option<std::process::Child>> = Mutex::new(None);
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FactoryConfig {
    pub shorts: bool,
    pub longform: bool,
    pub stitch: bool,
    pub publish: bool,
    pub local: bool,
    pub provider: String,
    pub cinematic: bool,
    pub specific_file: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ResearchTaskPayload {
    mode: String,
    query: String,
    success: bool,
    message: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct FactoryTaskPayload {
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiModelInfo {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "supportedGenerationMethods")]
    pub supported_generation_methods: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelListResponse {
    pub models: Vec<GeminiModelInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SentinelKeys {
    #[serde(deserialize_with = "deserialize_api_keys")]
    pub gemini_api_keys: Option<String>,
    pub perplexity_api_key: Option<String>,
    pub groq_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub gcloud_project_id: Option<String>,
    pub gcloud_region: Option<String>,
}

fn deserialize_api_keys<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let v: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Array(arr) => {
            let keys: Vec<String> = arr
                .into_iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect();
            if keys.is_empty() {
                Ok(None)
            } else {
                Ok(Some(keys.join(",")))
            }
        },
        _ => Ok(None),
    }
}

fn load_keys() -> SentinelKeys {
    let path = PathBuf::from(AGENTS_PATH).join("sentinel_keys.json");
    let mut keys = if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            SentinelKeys::default()
        }
    } else {
        SentinelKeys::default()
    };

    // Overrides de variables de entorno - soporta multiples llaves en GEMINI_API_KEYS
    // Overrides de entorno - fusiona env con archivo (prioriza env si el archivo falla)
    let env_gemini = std::env::var("GEMINI_API_KEYS")
        .ok()
        .or_else(|| std::env::var("GOOGLE_AI_API_KEY").ok())
        .or_else(|| std::env::var("GOOGLE_API_KEY").ok());

    if let Some(env_val) = env_gemini {
        match &mut keys.gemini_api_keys {
            Some(existing) => {
                if !existing.contains(&env_val) {
                    existing.push_str(",");
                    existing.push_str(&env_val);
                }
            },
            None => keys.gemini_api_keys = Some(env_val),
        }
    }
    if keys.perplexity_api_key.is_none() {
        keys.perplexity_api_key = std::env::var("PERPLEXITY_API_KEY").ok();
    }
    if keys.groq_api_key.is_none() {
        keys.groq_api_key = std::env::var("GROQ_API_KEY").ok();
    }
    if keys.openai_api_key.is_none() {
        keys.openai_api_key = std::env::var("OPENAI_API_KEY").ok();
    }
    if keys.gcloud_project_id.is_none() {
        keys.gcloud_project_id = std::env::var("GCLOUD_PROJECT_ID")
            .ok()
            .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok());
    }
    if keys.gcloud_region.is_none() {
        keys.gcloud_region = std::env::var("GCLOUD_REGION")
            .ok()
            .or_else(|| std::env::var("GOOGLE_CLOUD_LOCATION").ok());
    }

    keys
}

fn get_balanced_api_key(keys: &SentinelKeys, provider: &str) -> String {
    let (pool_opt, env_var) = match provider {
        "perplexity" => (&keys.perplexity_api_key, "PERPLEXITY_API_KEY"),
        "groq" => (&keys.groq_api_key, "GROQ_API_KEY"),
        "openai" => (&keys.openai_api_key, "OPENAI_API_KEY"),
        _ => (&keys.gemini_api_keys, "GOOGLE_AI_API_KEY"),
    };

    let pool_str = pool_opt.as_ref().map(|s| s.as_str()).unwrap_or("");
    let key_list: Vec<&str> = pool_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if key_list.is_empty() {
        return std::env::var(env_var).unwrap_or_default();
    }

    let index = API_KEY_COUNTER.fetch_add(1, Ordering::SeqCst) % key_list.len();
    key_list[index].to_string()
}

// ============================================================================
// SISTEMA DE COSTOS Y GESTION DE PRESUPUESTO
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiCall {
    pub timestamp: String,
    pub provider: String,
    pub endpoint: String,
    pub status_code: u16,
    pub latency_ms: u32,
    pub success: bool,
    pub error_message: Option<String>,
    pub cost_usd: f32,
    pub operation_type: String, // "video", "image", "text"
    pub efficiency_score: f32,
    pub gpu_load_at_call: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SentinomicaStats {
    pub requests: u32,
    pub errors: u32,
    pub last_status: String,
    pub last_used: String,
    // Campos para seguimiento de costos
    pub total_cost_usd: f32,
    pub cost_today_usd: f32,
    pub cost_this_month_usd: f32,
    pub avg_latency_ms: f32,
    pub p95_latency_ms: f32,
    pub success_rate: f32,
    pub quota_used: u32,
    pub quota_limit: u32,
    // Sentinomica: Seguimiento de ingresos
    pub revenue_today_usd: f32,
    pub revenue_this_month_usd: f32,
    pub total_revenue_usd: f32,
    // Normalizacion de eficiencia
    pub avg_efficiency_score: f32, // (ValorMercado / Costo) * (1 / Latencia)
    pub hardware_overhead_factor: f32, // Impacto en tiempo ocioso GPU/RAM
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UsoGlobalSentinomica {
    pub provider_stats: HashMap<String, SentinomicaStats>,
    pub recent_calls: Vec<ApiCall>, // Ultimas 100 llamadas
    pub daily_budget_usd: f32,
    pub monthly_budget_usd: f32,
    pub alert_threshold_pct: f32, // Alerta al alcanzar X% del presupuesto
    // Sentinomica: Ingresos globales
    pub total_revenue_usd: f32,
    pub daily_revenue_target_usd: f32,
    pub monthly_revenue_target_usd: f32,
}

// Tabla de precios (actualizada 2026-01-29)
#[allow(dead_code)]
fn get_operation_cost(operation_type: &str, provider: &str) -> f32 {
    match (provider, operation_type) {
        // Vertex AI - Veo 3
        ("vertex", "video-8s") => 0.25,
        ("vertex", "video-16s") => 0.50,
        ("vertex", "video-32s") => 1.00,
        // Vertex AI - Imagen 3
        ("vertex", "image") => 0.10,
        // Gemini API (por 1M tokens de entrada, estimado 1K por llamada)
        ("gemini", "text") => 0.0001,
        ("gemini", "flash") => 0.00005,
        // Perplexity
        ("perplexity", "search") => 0.005,
        // Infraestructura soberana - GPU local (costo cero)
        ("gpu", "render") => 0.0,
        ("gpu", "stitch") => 0.0,
        // Groq (tier gratis, se rastrea para futuro)
        ("groq", _) => 0.0,
        _ => 0.0,
    }
}

// Tabla de valor de mercado (ROI estimado)
fn get_operation_value(operation_type: &str, provider: &str) -> f32 {
    match (provider, operation_type) {
        // Activos de video de alta calidad
        ("vertex", "video-8s") => 5.00,
        ("vertex", "video-16s") => 12.00,
        ("vertex", "video-32s") => 25.00,
        // Activos de diseno
        ("vertex", "image") => 2.00,
        // Contenido estrategico (guiones, hilos, publicaciones)
        ("gemini", "text") => 1.50,
        ("gemini", "flash") => 0.75,
        ("perplexity", "search") => 0.50,
        _ => 0.10,
    }
}

fn log_api_usage(provider: &str, success: bool, status: &str) {
    log_api_call(provider, "unknown", status, 0, success, None, 0.0, "text");
}

fn log_api_call(
    provider: &str,
    endpoint: &str,
    status: &str,
    latency_ms: u32,
    success: bool,
    error_message: Option<String>,
    cost_usd: f32,
    operation_type: &str,
) {
    let path = PathBuf::from(AGENTS_PATH).join("sentinel_usage.json");
    let mut usage: UsoGlobalSentinomica = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        UsoGlobalSentinomica::default()
    };

    // Obtener carga actual de GPU
    let mut sys = System::new_all();
    sys.refresh_all();
    let gpu_load = 0.0;

    // Calcular puntaje de eficiencia: (ValorMercado / Costo) * (1 / Latencia)
    // Se controlan costo/latencia para evitar infinito
    let val = get_operation_value(operation_type, provider);
    let norm_cost = cost_usd.max(0.00001);
    let norm_latency = (latency_ms as f32 / 1000.0).max(0.1); // Normalizar a segundos, minimo 100ms
    let efficiency_score = (val / norm_cost) * (1.0 / norm_latency);

    // Crear registro de llamada API
    let call = ApiCall {
        timestamp: chrono::Local::now().to_rfc3339(),
        provider: provider.to_string(),
        endpoint: endpoint.to_string(),
        status_code: status.parse().unwrap_or(0),
        latency_ms,
        success,
        error_message,
        cost_usd,
        operation_type: operation_type.to_string(),
        efficiency_score,
        gpu_load_at_call: gpu_load,
    };

    // Agregar a llamadas recientes (mantener ultimas 100)
    usage.recent_calls.push(call);
    if usage.recent_calls.len() > 100 {
        usage.recent_calls.remove(0);
    }

    // Actualizar estadisticas por proveedor
    let entry = usage
        .provider_stats
        .entry(provider.to_string())
        .or_default();
    entry.requests += 1;
    if !success {
        entry.errors += 1;
    }
    entry.last_status = status.to_string();
    entry.last_used = chrono::Local::now().to_rfc3339();
    entry.total_cost_usd += cost_usd;

    // Calcular valor / ingresos
    let value_usd = if success {
        get_operation_value(operation_type, provider)
    } else {
        0.0
    };
    entry.total_revenue_usd += value_usd;
    usage.total_revenue_usd += value_usd;

    // Calcular estadisticas del dia
    let today = chrono::Local::now().date_naive();
    let today_calls: Vec<&ApiCall> = usage
        .recent_calls
        .iter()
        .filter(|c| {
            chrono::DateTime::parse_from_rfc3339(&c.timestamp)
                .ok()
                .map(|dt| dt.date_naive() == today)
                .unwrap_or(false)
        })
        .collect();

    entry.cost_today_usd = today_calls
        .iter()
        .filter(|c| c.provider == provider)
        .map(|c| c.cost_usd)
        .sum();

    entry.revenue_today_usd = today_calls
        .iter()
        .filter(|c| c.provider == provider && c.success)
        .map(|c| get_operation_value(&c.operation_type, &c.provider))
        .sum();

    // Calcular estadisticas del mes
    let this_month = chrono::Local::now().format("%Y-%m").to_string();
    let month_calls: Vec<&ApiCall> = usage
        .recent_calls
        .iter()
        .filter(|c| c.timestamp.starts_with(&this_month))
        .collect();

    entry.cost_this_month_usd = month_calls
        .iter()
        .filter(|c| c.provider == provider)
        .map(|c| c.cost_usd)
        .sum();

    entry.revenue_this_month_usd = month_calls
        .iter()
        .filter(|c| c.provider == provider && c.success)
        .map(|c| get_operation_value(&c.operation_type, &c.provider))
        .sum();

    // Calcular latencia promedio
    let provider_calls: Vec<&ApiCall> = usage
        .recent_calls
        .iter()
        .filter(|c| c.provider == provider)
        .collect();

    if !provider_calls.is_empty() {
        entry.avg_latency_ms = provider_calls
            .iter()
            .map(|c| c.latency_ms as f32)
            .sum::<f32>()
            / provider_calls.len() as f32;

        // Calcular latencia P95
        let mut latencies: Vec<u32> = provider_calls.iter().map(|c| c.latency_ms).collect();
        latencies.sort();
        let p95_idx = (latencies.len() as f32 * 0.95) as usize;
        entry.p95_latency_ms = latencies.get(p95_idx).copied().unwrap_or(0) as f32;

        // Calcular tasa de exito
        let success_count = provider_calls.iter().filter(|c| c.success).count();
        entry.success_rate = (success_count as f32 / provider_calls.len() as f32) * 100.0;

        // Calcular eficiencia promedio
        entry.avg_efficiency_score = provider_calls
            .iter()
            .map(|c| c.efficiency_score)
            .sum::<f32>()
            / provider_calls.len() as f32;

        // Calcular factor de sobrecarga de hardware
        // Basado en carga GPU mientras se espera respuesta
        entry.hardware_overhead_factor = provider_calls
            .iter()
            .map(|c| c.gpu_load_at_call * (c.latency_ms as f32 / 1000.0))
            .sum::<f32>()
            / provider_calls.len() as f32;
    }

    if let Ok(json) = serde_json::to_string_pretty(&usage) {
        let _ = fs::write(path, json);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatHistory {
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpEntry {
    pub id: String,
    pub status: String,
    pub target_file: String,
    pub gcs_uri: Option<String>,
    pub updated_at: Option<String>,
    pub op_type: String, // "scan" | "generate_short" | "generate_long" | "stitch" | "publish"
    pub engine: Option<String>, // "veo-3-fast" | "imagen-3"
    pub progress_pct: Option<f32>, // 0.0 - 100.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClaimStatus {
    pub name: String,
    pub status: String,
    pub active: bool,
    pub value: String,
    pub icon_type: String, // "truthsync" | "multimedia" | "ebpf" | "scanner"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VaultFile {
    pub name: String,
    pub path: String,
    pub modified_at: String, // ISO 8601 timestamp
    pub size_bytes: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CortexStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub uptime: u64,
    pub firewall_active: bool,
    pub logs_total: u32,
    pub kernel_version: String,
    pub swarm_load: f32,
    pub nervios_sync: bool,
    pub truthsync_audit: Option<TruthSyncAudit>,
    pub claims: Vec<ClaimStatus>,
    pub cpu_temp: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BalancerStatus {
    pub rag_index_path: String,
    pub rag_doc_count: usize,
    pub gpu_info: String,
    pub cpu_usage_pct: f32,
    pub memory_info: String,
    pub llm_cli_active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AgentState {
    Idle,
    Running,
    Error,
    Offline,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentStatus {
    pub name: String,
    pub state: AgentState,
    pub description: String,
    pub id: Option<u32>,
    pub binary: String,
    pub agent_type: String, // "nervio" | "research" | "cortex" | "media" | "generic"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResearchReport {
    pub title: String,
    pub path: String,
    pub content_preview: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TruthSyncSwitch {
    pub status: String,
    pub coherence: f32,
    pub truth_score: f32,
    pub timestamp: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TruthSyncAudit {
    pub timestamp: String,
    pub global_integrity: f32,
    pub switches: HashMap<String, TruthSyncSwitch>,
}

// --- UTILIDADES ---

fn check_lsmod_no_sudo(module: &str) -> bool {
    if let Ok(content) = fs::read_to_string("/proc/modules") {
        return content.contains(module);
    }
    false
}

// --- COMANDOS ---

#[tauri::command]
fn get_operaciones() -> Vec<OpEntry> {
    let ops_path = PathBuf::from(VAULT_PATH).join("SecurePenguin/.sentinel/operations.json");
    if let Ok(content) = fs::read_to_string(ops_path) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(ops) = data.get("operations").and_then(|o| o.as_array()) {
                return ops
                    .iter()
                    .map(|o| {
                        let status = o.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                        let target = o.get("target_file").and_then(|v| v.as_str()).unwrap_or("?");
                        let id = o
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?")
                            .to_string();
                        let op_type = if target.contains("_gen") {
                            "generate_short"
                        } else if target.contains("_final") {
                            "stitch"
                        } else if status.contains("Scanning") {
                            "scan"
                        } else {
                            "generate_long"
                        };

                        let engine = if id.contains("models/") {
                            id.split("models/")
                                .nth(1)
                                .and_then(|s| s.split('/').next())
                                .map(|s| s.to_string())
                        } else {
                            None
                        };

                        OpEntry {
                            id,
                            status: status.to_string(),
                            target_file: target.to_string(),
                            gcs_uri: o
                                .get("gcs_uri")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            updated_at: o
                                .get("updated_at")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            op_type: op_type.to_string(),
                            engine,
                            progress_pct: if status == "Done" { Some(100.0) } else { None },
                        }
                    })
                    .collect();
            }
        }
    }
    vec![]
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoProductionStats {
    pub total_operations: usize,
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub videos_ready_for_stitch: usize,
    pub avg_generation_time_mins: f32,
    pub active_vertex_projects: Vec<String>,
}

#[tauri::command]
fn get_estadisticas_fabrica() -> VideoProductionStats {
    let ops_path = PathBuf::from(VAULT_PATH).join("SecurePenguin/.sentinel/operations.json");
    let ops = if let Ok(content) = fs::read_to_string(&ops_path) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(ops_array) = data.get("operations").and_then(|o| o.as_array()) {
                ops_array.clone()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let total = ops.len();
    let pending = ops
        .iter()
        .filter(|o| o.get("status").and_then(|s| s.as_str()) == Some("Pending"))
        .count();
    let running = ops
        .iter()
        .filter(|o| o.get("status").and_then(|s| s.as_str()) == Some("Running"))
        .count();
    let completed = ops
        .iter()
        .filter(|o| {
            let status = o.get("status").and_then(|s| s.as_str()).unwrap_or("");
            status == "Done" || status == "Completed"
        })
        .count();
    let failed = ops
        .iter()
        .filter(|o| {
            let status = o.get("status").and_then(|s| s.as_str()).unwrap_or("");
            status.starts_with("Failed")
        })
        .count();

    let videos_ready = ops
        .iter()
        .filter(|o| {
            let status = o.get("status").and_then(|s| s.as_str()).unwrap_or("");
            let target = o.get("target_file").and_then(|t| t.as_str()).unwrap_or("");
            (status == "Done" || status == "Completed") && target.ends_with("_gen.mp4")
        })
        .count();

    let avg_time = if completed > 0 { 2.0 } else { 0.0 };

    let vertex_projects = vec!["fenix-sentinel-core".to_string()];

    VideoProductionStats {
        total_operations: total,
        pending,
        running,
        completed,
        failed,
        videos_ready_for_stitch: videos_ready,
        avg_generation_time_mins: avg_time,
        active_vertex_projects: vertex_projects,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadyEntry {
    pub file: String,
    pub score: f32,
    pub status: String,
}

fn get_ready_entries() -> Vec<ReadyEntry> {
    let ready_path = PathBuf::from(AGENTS_PATH).join("ready.json");
    if let Ok(content) = fs::read_to_string(ready_path) {
        if let Ok(entries) = serde_json::from_str::<Vec<ReadyEntry>>(&content) {
            return entries;
        }
    }
    vec![]
}

#[tauri::command]
fn get_estadisticas_cortex() -> CortexStats {
    let mut sys = System::new_all();
    sys.refresh_all();

    let firewall_active = PathBuf::from("/etc/nftables.conf").exists();
    let vault_files = get_archivos_sentinel_media();
    let ready_entries = get_ready_entries();
    let ops = get_operaciones();

    // Leer TruthSync Audit Real
    let truthsync_path = PathBuf::from(AGENTS_PATH).join("TRUTHSYNC_MASTER_AUDIT.json");
    let truthsync_audit = if let Ok(content) = fs::read_to_string(truthsync_path) {
        serde_json::from_str::<TruthSyncAudit>(&content).ok()
    } else {
        None
    };

    let unison_count = ready_entries
        .iter()
        .filter(|e| e.status == "UNISON")
        .count();
    let avg_score = if !ready_entries.is_empty() {
        ready_entries.iter().map(|e| e.score).sum::<f32>() / ready_entries.len() as f32
    } else {
        0.0
    };

    let has_ebpf = check_lsmod_no_sudo("bpf") || PathBuf::from("/sys/fs/bpf").exists();
    let agents = get_agentes();
    let running_agents = agents
        .iter()
        .filter(|a| a.state == AgentState::Running)
        .count();

    let claims = vec![
        ClaimStatus {
            name: "TruthSync Status".into(),
            status: format!("{} Docs Validados", unison_count),
            active: unison_count > 0,
            value: format!("{}/{}", unison_count, vault_files.len()),
            icon_type: "truthsync".into(),
        },
        ClaimStatus {
            name: "Data Integrity".into(),
            status: "Media Score (RAG)".into(),
            active: avg_score > 0.9,
            value: format!("{:.4}", avg_score),
            icon_type: "scanner".into(),
        },
        ClaimStatus {
            name: "Factory Ops".into(),
            status: format!(
                "{} en proceso",
                ops.iter().filter(|o| o.status == "Running").count()
            ),
            active: !ops.is_empty(),
            value: format!("{} OPs", ops.len()),
            icon_type: "multimedia".into(),
        },
        ClaimStatus {
            name: "Scanner State".into(),
            status: "Sync Status".into(),
            active: !ready_entries.is_empty(),
            value: if !ready_entries.is_empty() {
                "READY"
            } else {
                "WAIT"
            }
            .into(),
            icon_type: "scanner".into(),
        },
        ClaimStatus {
            name: "Kernel Security".into(),
            status: if has_ebpf {
                "LSM Activo".into()
            } else {
                "Standby".into()
            },
            active: has_ebpf,
            value: if has_ebpf { "PROT" } else { "OFF" }.into(),
            icon_type: "ebpf".into(),
        },
    ];

    let cpu_temp = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .ok()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|t| t / 1000.0)
        .unwrap_or(0.0);

    CortexStats {
        cpu_usage: sys.global_cpu_info().cpu_usage(),
        memory_used: sys.used_memory(),
        memory_total: sys.total_memory(),
        uptime: System::uptime(),
        firewall_active,
        logs_total: vault_files.len() as u32,
        kernel_version: System::kernel_version().unwrap_or_else(|| "Debian".into()),
        swarm_load: running_agents as f32 / agents.len().max(1) as f32,
        nervios_sync: running_agents >= 1,
        truthsync_audit, // Telemetría Real
        claims,
        cpu_temp,
    }
}

#[tauri::command]
fn get_archivos_sentinel_media() -> Vec<VaultFile> {
    let vault_path = PathBuf::from(VAULT_PATH).join("SecurePenguin");
    if let Ok(entries) = fs::read_dir(vault_path) {
        return entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file() && e.path().extension().map_or(false, |ext| ext == "md"))
            .map(|e| {
                let metadata = e.metadata().ok();
                let modified_at = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(|t| chrono::DateTime::<chrono::Local>::from(t).to_rfc3339())
                    .unwrap_or_default();
                let size_bytes = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

                VaultFile {
                    name: e.file_name().to_string_lossy().to_string(),
                    path: e.path().to_string_lossy().to_string(),
                    modified_at,
                    size_bytes,
                }
            })
            .collect();
    }
    vec![]
}

#[tauri::command]
fn get_reportes_investigacion() -> Vec<ResearchReport> {
    let research_path = PathBuf::from("vault");
    let mut reports = vec![];
    if let Ok(entries) = fs::read_dir(research_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                if let Ok(content) = fs::read_to_string(&path) {
                    reports.push(ResearchReport {
                        title: path
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_else(|| "reporte_sin_nombre".to_string()),
                        path: path.to_string_lossy().to_string(),
                        content_preview: content.chars().take(200).collect(),
                    });
                }
            }
        }
    }
    reports
}

#[tauri::command]
fn get_estado_balanceador() -> BalancerStatus {
    let ready_path = PathBuf::from(AGENTS_PATH).join("ready.json");
    let (rag_index_path, rag_doc_count) = if let Ok(content) = fs::read_to_string(&ready_path) {
        if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            (ready_path.to_string_lossy().to_string(), entries.len())
        } else {
            ("Error parseando ready.json".into(), 0)
        }
    } else {
        ("Archivo ready.json no encontrado".into(), 0)
    };

    // Telemetria real de GPU
    let gpu_info = Command::new("nvidia-smi")
        .arg("--query-gpu=name")
        .arg("--format=csv,noheader")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "No se detectó GPU NVIDIA (CUDA)".into());

    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu_usage_pct = sys.global_cpu_info().cpu_usage();
    let memory_info = format!(
        "{:.1} / {:.1} GiB",
        sys.used_memory() as f32 / (1024 * 1024 * 1024) as f32,
        sys.total_memory() as f32 / (1024 * 1024 * 1024) as f32
    );

    // LLM CLI activo?
    let llm_cli_active = sys
        .processes()
        .values()
        .any(|p| p.name().contains("sentinel_cli"));

    BalancerStatus {
        rag_index_path,
        rag_doc_count,
        gpu_info,
        cpu_usage_pct,
        memory_info,
        llm_cli_active,
    }
}

#[tauri::command]
async fn get_crystal_resonance() -> Result<Vec<f32>, String> {
    let crystal_path = Path::new("/var/lib/pai60/memory.crystal");
    if !crystal_path.exists() {
        return Ok(vec![0.0; 12]); // Fallback si no hay Ring 0 activo
    }

    let store = CrystalStore::open(crystal_path, 12)
        .map_err(|e| format!("Error abriendo Crystal Store: {}", e))?;
    
    let lattice = store.load();
    let amplitudes: Vec<f32> = lattice.crystals
        .iter()
        .map(|c| c.amplitude.to_raw() as f32 / 1_000_000.0) // Aproximación decimal para la GUI
        .collect();

    Ok(amplitudes)
}

#[tauri::command]
async fn get_balancer_status() -> Result<String, String> {
        .map_err(|e| e.to_string())?;

    let project = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !project.is_empty() {
        Ok(format!("NOMINAL ({})", project))
    } else {
        Ok("ADVERTENCIA: No se detecto proyecto de GCloud".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResearchAgentStatus {
    pub state: String,
    pub rag_loaded: bool,
    pub rag_doc_count: usize,
    pub web_search_ready: bool,
    pub translators_active: bool,
}

#[tauri::command]
fn get_estado_agente_investigacion() -> ResearchAgentStatus {
    let ready_path = PathBuf::from(AGENTS_PATH).join("ready.json");
    let (rag_loaded, rag_doc_count) = if ready_path.exists() {
        if let Ok(content) = fs::read_to_string(&ready_path) {
            if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                (true, entries.len())
            } else {
                (false, 0)
            }
        } else {
            (false, 0)
        }
    } else {
        (false, 0)
    };

    // Verificar si el proceso sentinel_research esta en ejecucion
    let mut sys = System::new_all();
    sys.refresh_all();
    let research_running = sys
        .processes()
        .values()
        .any(|p| p.name().contains("sentinel_research"));

    // La busqueda web esta lista si hay llaves API
    let keys = load_keys();
    let web_search_ready = keys.gemini_api_keys.is_some() || keys.perplexity_api_key.is_some();

    // Traductores activos si el agente de investigacion esta corriendo
    let translators_active = research_running;

    let state = if research_running {
        "RUNNING".to_string()
    } else if rag_loaded {
        "IDLE".to_string()
    } else {
        "OFFLINE".to_string()
    };

    ResearchAgentStatus {
        state,
        rag_loaded,
        rag_doc_count,
        web_search_ready,
        translators_active,
    }
}

// Struct para logs estructurados
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemLog {
    timestamp: String,
    level: String,
    message: String,
}

#[tauri::command]
fn get_logs_sistema(count: usize) -> Vec<crate::services::log_streamer::ProcessedLog> {
    use crate::services::log_streamer::{LogSeverity, ProcessedLog};
    use std::time::SystemTime;

    let output = Command::new("journalctl")
        .args(&["-n", &count.to_string(), "--output=cat", "--no-pager"])
        .output();

    if let Ok(out) = output {
        let log_text = String::from_utf8_lossy(&out.stdout);
        let mut logs = Vec::new();

        for line in log_text.lines() {
            let upper = line.to_uppercase();
            let severity = if upper.contains("ERROR")
                || upper.contains("FATAL")
                || upper.contains("PANIC")
            {
                LogSeverity::Critical
            } else if upper.contains("WARN") {
                LogSeverity::Warning
            } else if (upper.contains("GPU") || upper.contains("TEMP") || upper.contains("THERMAL"))
                && upper.contains("CRITICAL")
            {
                LogSeverity::HardwareAlert
            } else if upper.contains("SENTINEL") || upper.contains("CORTEX") {
                LogSeverity::Info
            } else {
                continue;
            };

            logs.push(ProcessedLog {
                message: line.to_string(),
                severity,
                timestamp: SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(), // Timestamp aproximado, mejor usar del log si posible
            });
        }
        logs.reverse(); // Mostrar más recientes arriba
        return logs;
    }
    vec![]
}

#[tauri::command]
fn get_agentes() -> Vec<AgentStatus> {
    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
    sys.refresh_processes();

    // ACTUALIZACION ADN: carga dinamica desde swarm_manifest.json
    let manifest_path = PathBuf::from(AGENTS_PATH).join("swarm_manifest.json");

    if let Ok(content) = fs::read_to_string(&manifest_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(agents_array) = json["agents"].as_array() {
                let dynamic_agents: Vec<AgentStatus> = agents_array
                    .iter()
                    .filter_map(|agent| {
                        let name = agent["name"].as_str().unwrap_or("Desconocido");
                        let id = agent["id"].as_str().unwrap_or("");
                        let desc = agent["description"].as_str().unwrap_or("");
                        // Mapear icono a agent_type para la GUI
                        let icon = agent["icon"].as_str().unwrap_or("cpu");
                        let agent_type = match icon {
                            "microscope" => "research",
                            "film" => "media",
                            "terminal" => "cortex",
                            "cpu" => "nervio",
                            _ => "cortex",
                        };

                        // Extraer nombre de binario para matcheo de proceso
                        let binary_path_str = agent["binary_path"].as_str().unwrap_or("");
                        let binary_name = Path::new(binary_path_str).file_name()?.to_str()?;

                        // Verificar estado del proceso
                        let process = sys
                            .processes()
                            .values()
                            .find(|p| p.name().contains(binary_name));
                        let state = if process.is_some() {
                            AgentState::Running
                        } else {
                            AgentState::Idle
                        };

                        Some(AgentStatus {
                            name: name.into(),
                            state,
                            description: desc.into(),
                            id: process.map(|p| p.pid().as_u32()),
                            binary: id.into(), // Pasar ID a iniciar_agente
                            agent_type: agent_type.into(),
                        })
                    })
                    .collect();

                if !dynamic_agents.is_empty() {
                    return dynamic_agents;
                }
            }
        }
    }

    // FALLBACK LEGADO
    let agents_config = vec![
        (
            "Cortex Core",
            "sentinel_cli",
            "Motor central de decisiones",
            "cortex",
        ),
        (
            "System Agent",
            "sentinel_system",
            "Gestión de sistema",
            "nervio",
        ),
        (
            "Research PAI",
            "sentinel_research",
            "Síntesis de conocimiento",
            "research",
        ),
        (
            "Media Hub",
            "sentinel_media",
            "Producción multimedia",
            "media",
        ),
        (
            "Gemini AI CLI",
            "gemini",
            "Gestión de sesiones y workspaces",
            "gemini",
        ),
    ];

    agents_config
        .into_iter()
        .filter_map(|(name, binary, desc, a_type)| {
            let is_system_cmd = binary == "gemini";

            if !is_system_cmd {
                let binary_path = PathBuf::from(AGENTS_PATH)
                    .join(format!("{}/target/release/{}", binary, binary));
                if !binary_path.exists() {
                    return None;
                }
            }

            let process = sys.processes().values().find(|p| p.name().contains(binary));
            let state = if process.is_some() {
                AgentState::Running
            } else {
                AgentState::Idle
            };
            Some(AgentStatus {
                name: name.into(),
                state,
                description: desc.into(),
                id: process.map(|p| p.pid().as_u32()),
                binary: binary.into(),
                agent_type: a_type.into(),
            })
        })
        .collect()
}

#[tauri::command]
fn iniciar_agente(binary: String) -> Result<String, String> {
    // ACTUALIZACION ADN: cargar agentes desde swarm_manifest.json
    let manifest_path = PathBuf::from(AGENTS_PATH).join("swarm_manifest.json");
    let binary_path = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let agents = json["agents"]
            .as_array()
            .ok_or("Formato de manifest invalido")?;
        let agent = agents
            .iter()
            .find(|a| a["id"].as_str() == Some(binary.as_str()));

        if let Some(a) = agent {
            PathBuf::from(AGENTS_PATH).join(a["binary_path"].as_str().unwrap_or_default())
        } else {
            // Respaldo para compatibilidad legado o agentes no definidos
            match binary.as_str() {
                "sentinel_cli" => {
                    PathBuf::from(AGENTS_PATH).join("sentinel_cli/target/release/sentinel_cli")
                },
                "sentinel_system" => PathBuf::from(AGENTS_PATH)
                    .join("sentinel_system/target/release/sentinel_system"),
                "sentinel_research" => PathBuf::from(AGENTS_PATH)
                    .join("sentinel_research/target/release/sentinel_research"),
                "sentinel_media" => {
                    PathBuf::from(AGENTS_PATH).join("sentinel_media/target/release/sentinel_media")
                },
                _ => return Err(format!("Agente no registrado en Enjambre: {}", binary)),
            }
        }
    } else {
        // Respaldo si falta el manifest
        match binary.as_str() {
            "sentinel_cli" => {
                PathBuf::from(AGENTS_PATH).join("sentinel_cli/target/release/sentinel_cli")
            },
            "sentinel_system" => {
                PathBuf::from(AGENTS_PATH).join("sentinel_system/target/release/sentinel_system")
            },
            "sentinel_research" => PathBuf::from(AGENTS_PATH)
                .join("sentinel_research/target/release/sentinel_research"),
            "sentinel_media" => {
                PathBuf::from(AGENTS_PATH).join("sentinel_media/target/release/sentinel_media")
            },
            _ => return Err(format!("Binario desconocido (Legacy): {}", binary)),
        }
    };

    // Verificar que el binario exista
    if !binary_path.exists() {
        return Err(format!("Binario no encontrado: {:?}", binary_path));
    }

    let child = Command::new(&binary_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("No se pudo iniciar {}: {}", binary, e))?;

    Ok(child.id().to_string())
}

#[tauri::command]
fn detener_agente(pid: u32) -> Result<String, String> {
    let mut sys = System::new_all();
    sys.refresh_processes();
    if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
        process.kill();
        return Ok("Agente detenido correctamente".to_string());
    }
    Err("Proceso no encontrado".into())
}

#[tauri::command]
async fn send_neural_message(messages: Vec<ChatMessage>, model_id: Option<String>) -> String {
    // 1. Contexto RAG y Misión
    let ready_path = PathBuf::from(AGENTS_PATH).join("ready.json");
    let rag_context = if ready_path.exists() {
        match fs::read_to_string(&ready_path) {
            Ok(content) => {
                let entries: Result<Vec<ReadyEntry>, _> = serde_json::from_str(&content);
                match entries {
                    Ok(files) => {
                        let context_files: Vec<String> = files
                            .iter()
                            .take(10)
                            .map(|e| format!("- {} (Score: {})", e.file, e.score))
                            .collect();
                        format!(
                            "Archivos indexados recientes en la bóveda:\n{}",
                            context_files.join("\n")
                        )
                    },
                    Err(_) => String::new(),
                }
            },
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    // 2. Cargar Misión/Contexto desde ANTIGRAVITY_CONTEXT.md y SENTINEL_PROMPT.md
    let base_dir = PathBuf::from(AGENTS_PATH);
    let sentinel_prompt_path = base_dir.join("SENTINEL_PROMPT.md");
    let antigravity_context_path = base_dir.join("ANTIGRAVITY_CONTEXT.md");

    let mut base_mission = String::new();

    if sentinel_prompt_path.exists() {
        base_mission.push_str(&fs::read_to_string(&sentinel_prompt_path).unwrap_or_default());
        base_mission.push_str("\n\n");
    }

    if antigravity_context_path.exists() {
        base_mission.push_str(&fs::read_to_string(&antigravity_context_path).unwrap_or_default());
    }

    if base_mission.is_empty() {
        base_mission = "Interfaz técnica de gestión Sentinel. ADVERTENCIA: Contextos de Bóveda no encontrados.".to_string();
    }

    // 3. CONSULTA DE MEMORIA NEURONAL (RAG) - BASADO EN SCV
    // Ejecutar 'sentinel_memory query' con el último mensaje del usuario para coherencia semántica
    let last_user_msg = messages
        .iter()
        .rfind(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();
    let memory_context = if !last_user_msg.is_empty() {
        let mem_bin =
            PathBuf::from(AGENTS_PATH).join("sentinel_memory/target/release/sentinel_memory");
        if std::path::Path::new(&mem_bin).exists() {
            match std::process::Command::new(&mem_bin)
                .arg("query")
                .arg(&last_user_msg)
                .output()
            {
                Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
                Err(_) => String::new(),
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let system_prompt = format!(
        "{}\n\n[MEMORIA NEURONAL - RESULTADOS SCV]:\n{}\n\n[ARCHIVOS RECIENTES]:\n{}",
        base_mission, memory_context, rag_context
    );

    let model = model_id
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "gemini-2.5-flash".to_string());

    let client = reqwest::Client::new();
    let keys = load_keys();

    // =========================================================================================
    // ROUTER DE PROVEEDORES
    // =========================================================================================

    // Excluir Vertex (publishers/) y Sentinel (antigravity/cli) del bloque de Keys
    if (model.starts_with("gemini") || model.contains("google"))
        && !model.contains("publishers")
        && !model.contains("antigravity")
        && !model.contains("sentinel")
    {
        // --- GOOGLE GEMINI / VERTEX (balanceador inteligente con conmutacion) ---
        // 1. Obtener todas las keys disponibles
        let keys_env = keys.gemini_api_keys.clone().unwrap_or_default();
        let all_keys: Vec<String> = keys_env
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if all_keys.is_empty() {
            return "Error: No hay API Keys de Google (GEMINI_API_KEYS)".to_string();
        }

        // 2. Determinar punto de inicio balanceado (Round-Robin)
        let total_keys = all_keys.len();
        let start_idx = API_KEY_COUNTER.fetch_add(1, Ordering::Relaxed) % total_keys;

        // 3. Crear lista ordenada empezando por la key balanceada, seguida del resto (anillo de conmutacion)
        let mut available_keys = Vec::with_capacity(total_keys);
        for i in 0..total_keys {
            let idx = (start_idx + i) % total_keys;
            available_keys.push(all_keys[idx].clone());
        }

        // ============================================================================
        // ROUND-ROBIN LOAD BALANCER (3 Cuentas Empresariales Independientes)
        // ============================================================================
        let total_keys = available_keys.len();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model
        );
        let mut last_error = String::from("Error desconocido");

        // Intentar hasta 3 veces (una por cada key)
        for attempt in 0..total_keys {
            // Obtener índice actual y avanzar el contador atómicamente
            let current_idx = API_KEY_COUNTER.fetch_add(1, Ordering::Relaxed) % total_keys;
            let api_key = &available_keys[current_idx];

            eprintln!(
                "[Sentinel Router] Intento {}/{} usando clave #{}",
                attempt + 1,
                total_keys,
                current_idx + 1
            );
            let contents: Vec<serde_json::Value> = messages
                .iter()
                .map(|msg| {
                    let role = if msg.role == "user" { "user" } else { "model" };
                    serde_json::json!({ "role": role, "parts": [{ "text": msg.content }] })
                })
                .collect();

            let body = serde_json::json!({
                "contents": contents,
                "system_instruction": {
                    "parts": [
                        { "text": system_prompt }
                    ]
                }
            });

            let res = client
                .post(format!("{}?key={}", url, api_key))
                .json(&body)
                .send()
                .await;

            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let json: serde_json::Value = resp.json().await.unwrap_or_default();
                        let text = json["candidates"][0]["content"]["parts"][0]["text"]
                            .as_str()
                            .unwrap_or("Error: El modelo no devolvió texto. Revise el historial o el prompt.")
                            .to_string();

                        log_api_usage("gemini", true, "OK");
                        eprintln!("[Sentinel Router] ✅ Exito con clave #{}", current_idx + 1);

                        // --- BUCLE DE USO DE HERRAMIENTAS (Nexus) ---
                        if text.contains("[EXEC:") {
                            if let (Some(start), Some(end)) = (text.find("[EXEC:"), text.find(']'))
                            {
                                let cmd_str = &text[start + 6..end].trim();

                                // Ejecutar comando
                                let output = tokio::process::Command::new("sh")
                                    .arg("-c")
                                    .arg(cmd_str)
                                    .output()
                                    .await;
                                let result_text = match output {
                                    Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
                                    Err(e) => format!("Error: {}", e),
                                };

                                let mut mutable_messages = messages.clone();
                                mutable_messages.push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: text.clone(),
                                    timestamp: "".to_string(),
                                });
                                mutable_messages.push(ChatMessage {
                                    role: "user".to_string(),
                                    content: format!("OBSERVACIÓN: {}", result_text),
                                    timestamp: "".to_string(),
                                });

                                return Box::pin(send_neural_message(
                                    mutable_messages,
                                    Some(model.clone()),
                                ))
                                .await;
                            }
                        }

                        return text;
                    }
                    // ❌ Fallo esta key (429, 400, etc.) → Saltar a la siguiente
                    let status_val = resp.status();
                    let error_text = resp.text().await.unwrap_or_default();
                    last_error = format!(
                        "Clave #{} fallo ({}) - {}",
                        current_idx + 1,
                        status_val,
                        error_text
                    );
                    log_api_usage("gemini", false, &status_val.to_string());
                    eprintln!("[Sentinel Router] ❌ {}", last_error);
                    continue;
                },
                Err(e) => {
                    last_error = format!("Clave #{} error de conexion: {}", current_idx + 1, e);
                    log_api_usage("gemini", false, "ConnErr");
                    eprintln!("[Sentinel Router] ❌ {}", last_error);
                },
            }
        }

        return format!(
            "Todas las cuentas de Google fallaron. Último error: {}",
            last_error
        );
    } else if model.contains("groq") || model.contains("llama3") || model.contains("mixtral") {
        // --- GROQ CLOUD ---
        let api_key = get_balanced_api_key(&keys, "groq");
        if api_key.is_empty() {
            return "Error: Falta API Key para Groq".to_string();
        }

        let endpoint = "https://api.groq.com/openai/v1/chat/completions";
        let body = serde_json::json!({
            "model": model,
            "messages": messages.iter().map(|msg| {
                serde_json::json!({ "role": msg.role, "content": msg.content })
            }).collect::<Vec<_>>(),
            "temperature": 0.7
        });

        let res = client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await;

        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json: serde_json::Value = resp.json().await.unwrap_or_default();
                    log_api_usage("groq", true, "OK");
                    json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("Error al parsear respuesta de Groq")
                        .to_string()
                } else {
                    let st = resp.status().to_string();
                    log_api_usage("groq", false, &st);
                    format!(
                        "Error de Groq {}: {}",
                        st,
                        resp.text().await.unwrap_or_default()
                    )
                }
            },
            Err(e) => {
                log_api_usage("groq", false, "ConnErr");
                format!("Error de conexion con Groq: {}", e)
            },
        }
    } else if model.contains("perplexity") || model.contains("sonar") {
        // --- PERPLEXITY AI ---
        let api_key = get_balanced_api_key(&keys, "perplexity");
        if api_key.is_empty() {
            return "Error: Falta API Key para Perplexity".to_string();
        }

        let endpoint = "https://api.perplexity.ai/chat/completions";
        let body = serde_json::json!({
            "model": if model.contains("sonar") { &model } else { "sonar-pro" },
            "messages": messages.iter().map(|msg| {
                serde_json::json!({ "role": msg.role, "content": msg.content })
            }).collect::<Vec<_>>()
        });

        let res = client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await;

        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json: serde_json::Value = resp.json().await.unwrap_or_default();
                    log_api_usage("perplexity", true, "OK");
                    json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("Error al parsear respuesta de Perplexity")
                        .to_string()
                } else {
                    let st = resp.status().to_string();
                    log_api_usage("perplexity", false, &st);
                    format!(
                        "Error de Perplexity {}: {}",
                        st,
                        resp.text().await.unwrap_or_default()
                    )
                }
            },
            Err(e) => {
                log_api_usage("perplexity", false, "ConnErr");
                format!("Error de conexion con Perplexity: {}", e)
            },
        }
    } else if model.starts_with("gpt-") || model.contains("openai") {
        // --- OPENAI PRO ---
        let api_key = get_balanced_api_key(&keys, "openai");
        if api_key.is_empty() {
            return "Error: Falta API Key para OpenAI".to_string();
        }

        let endpoint = "https://api.openai.com/v1/chat/completions";
        let body = serde_json::json!({
            "model": model,
            "messages": messages.iter().map(|msg| {
                serde_json::json!({ "role": msg.role, "content": msg.content })
            }).collect::<Vec<_>>()
        });

        let res = client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await;

        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json: serde_json::Value = resp.json().await.unwrap_or_default();
                    log_api_usage("openai", true, "OK");
                    json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("Error al parsear respuesta de OpenAI")
                        .to_string()
                } else {
                    let st = resp.status().to_string();
                    log_api_usage("openai", false, &st);
                    format!(
                        "Error de OpenAI {}: {}",
                        st,
                        resp.text().await.unwrap_or_default()
                    )
                }
            },
            Err(e) => {
                log_api_usage("openai", false, "ConnErr");
                format!("Error de conexion con OpenAI: {}", e)
            },
        }
    } else if model.starts_with("vertex-") || model.contains("publishers/") {
        // --- GOOGLE VERTEX AI ---
        let project = keys.gcloud_project_id.clone().unwrap_or_default();
        let region = keys
            .gcloud_region
            .clone()
            .unwrap_or_else(|| "us-central1".to_string());

        if project.is_empty() {
            return "Error: GCLOUD_PROJECT_ID requerido para modelos Vertex (configurar en env o sentinel_keys.json)".to_string();
        }

        // Bloquear proyectos de AI Studio (gen-lang-client-*) que no son compatibles con Vertex AI
        if project.starts_with("gen-lang-client-") {
            return format!("Error: El proyecto '{}' es una cuenta de 'Google AI Studio' y no soporta el endpoint de 'Vertex AI'. Use modelos estándar (gemini-*) o desactive el Project ID.", project);
        }

        // Obtener access token via shell
        let token_output = Command::new("gcloud")
            .arg("auth")
            .arg("print-access-token")
            .output();

        let token = match token_output {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            },
            _ => {
                return "Error obteniendo token de gcloud (fallo gcloud auth print-access-token)"
                    .to_string()
            },
        };

        let clean_model = if model.contains("publishers/") {
            model
                .trim_start_matches("publishers/google/models/")
                .to_string()
        } else {
            model.replace("vertex-", "")
        };

        let url = format!("https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent", 
            region, project, region, clean_model);

        let contents: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                let role = if msg.role == "user" { "user" } else { "model" };
                serde_json::json!({ "role": role, "parts": [{ "text": msg.content }] })
            })
            .collect();

        let body = serde_json::json!({
            "contents": contents,
            "system_instruction": { "parts": [{ "text": system_prompt }] }
        });

        let res = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await;

        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json: serde_json::Value = resp.json().await.unwrap_or_default();
                    log_api_usage("vertex", true, "OK");
                    json["candidates"][0]["content"]["parts"][0]["text"]
                        .as_str()
                        .unwrap_or("Error al parsear respuesta de Vertex")
                        .to_string()
                } else {
                    let st = resp.status().to_string();
                    log_api_usage("vertex", false, &st);
                    format!(
                        "Error de API Vertex {}: {}",
                        st,
                        resp.text().await.unwrap_or_default()
                    )
                }
            },
            Err(e) => {
                log_api_usage("vertex", false, "ConnErr");
                format!("Error de conexion: {}", e)
            },
        }
    } else if model.contains("antigravity") || model.contains("sentinel") {
        // --- SENTINEL NATIVE BRIDGE ---
        // --- SENTINEL NATIVE BRIDGE ---
        let user_msg = messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        // Inyectar system prompt + RAG + historial (simplificado)
        let full_context = format!("{}\n\nINPUT USUARIO:\n{}", system_prompt, user_msg);

        let output = tokio::process::Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"))
            .arg("predict")
            .arg(&full_context)
            .output()
            .await;

        match output {
            Ok(o) => {
                if o.status.success() {
                    String::from_utf8_lossy(&o.stdout).to_string()
                } else {
                    format!(
                        "Error de Sentinel CLI: {}",
                        String::from_utf8_lossy(&o.stderr)
                    )
                }
            },
            Err(e) => format!(
                "Error de puente: {}\nVerifique que el binario 'sentinel' exista en la ruta.",
                e
            ),
        }
    } else {
        // --- COMPATIBLE CON OPENAI (Perplexity, Groq, Grok, OpenAI) ---
        let (api_key, url) = if model.contains("sonar") {
            (
                keys.perplexity_api_key.clone().unwrap_or_default(),
                "https://api.perplexity.ai/chat/completions",
            )
        } else if model.contains("grok") {
            // Asumiendo key de xAI o Groq
            (
                keys.groq_api_key.clone().unwrap_or_default(),
                "https://api.x.ai/v1/chat/completions",
            )
        } else if model.contains("llama") || model.contains("mixtral") || model.contains("gemma") {
            (
                keys.groq_api_key.clone().unwrap_or_default(),
                "https://api.groq.com/openai/v1/chat/completions",
            )
        } else {
            (
                keys.openai_api_key.clone().unwrap_or_default(),
                "https://api.openai.com/v1/chat/completions",
            )
        };

        if api_key.is_empty() {
            return format!(
                "Error: Falta API Key para el modelo {} en sentinel_keys.json o env",
                model
            );
        }

        let mut openai_messages = Vec::new();
        openai_messages.push(serde_json::json!({ "role": "system", "content": system_prompt }));
        for msg in messages {
            let role = if msg.role == "user" {
                "user"
            } else {
                "assistant"
            };
            openai_messages.push(serde_json::json!({ "role": role, "content": msg.content }));
        }

        let body = serde_json::json!({
            "model": model,
            "messages": openai_messages,
            "max_tokens": 4096 // Limite generoso
        });

        let res = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        match res {
            Ok(resp) => {
                let provider_name = if model.starts_with("sonar") {
                    "perplexity"
                } else if model.starts_with("gemma") || model.starts_with("llama") {
                    "groq"
                } else {
                    "openai"
                };
                if resp.status().is_success() {
                    let json: serde_json::Value = resp.json().await.unwrap_or_default();
                    log_api_usage(provider_name, true, "OK");
                    json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("Error al parsear respuesta con formato OpenAI")
                        .to_string()
                } else {
                    let st = resp.status().to_string();
                    log_api_usage(provider_name, false, &st);
                    format!(
                        "Error de API {}: {}",
                        st,
                        resp.text().await.unwrap_or_default()
                    )
                }
            },
            Err(e) => {
                log_api_usage("openai_compatible", false, "ConnErr");
                format!("Error de conexion: {}", e)
            },
        }
    }
}

#[tauri::command]
fn get_llaves_api() -> SentinelKeys {
    load_keys()
}

#[tauri::command]
fn guardar_llaves_api(keys: SentinelKeys) -> Result<String, String> {
    let path = PathBuf::from(AGENTS_PATH).join("sentinel_keys.json");
    let json = serde_json::to_string_pretty(&keys).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok("Llaves guardadas correctamente".to_string())
}

#[tauri::command]
fn obtener_uso_sentinomica() -> UsoGlobalSentinomica {
    let path = PathBuf::from(AGENTS_PATH).join("sentinel_usage.json");
    if path.exists() {
        fs::read_to_string(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        UsoGlobalSentinomica::default()
    }
}

#[tauri::command]
async fn autenticar_gcloud() -> Result<String, String> {
    // Usamos kitty para mostrar el proceso interactivo de autenticación
    let _ = Command::new("kitty")
        .arg("-e")
        .arg("gcloud")
        .arg("auth")
        .arg("login")
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok("Autenticación iniciada en terminal externa.".to_string())
}

#[tauri::command]
async fn leer_contenido_archivo_sentinel_media(path: String) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn obtener_modelos_disponibles() -> Result<Vec<String>, String> {
    let mut models = vec![
        "google-antigravity-native".to_string(),
        "sentinel-cli-bridge".to_string(),
    ];
    let keys = load_keys();
    let client = reqwest::Client::new();

    // 1. Google Gemini (Balanceador de Tokens para Descubrimiento)
    if let Some(key_str) = &keys.gemini_api_keys {
        let available_keys: Vec<String> = key_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        for api_key in available_keys {
            let res = client
                .get(format!(
                    "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                    api_key
                ))
                .send()
                .await;

            if let Ok(resp) = res {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<ModelListResponse>().await {
                        let google_models: Vec<String> = data.models.into_iter()
                            .filter(|m| m.supported_generation_methods.as_ref().map(|v| v.contains(&"generateContent".to_string())).unwrap_or(false))
                            // Preferimos modelos modernos (2.0+) pero mostramos lo que la API reporte como disponible
                            .map(|m| m.name.replace("models/", ""))
                            .collect();
                        models.extend(google_models);
                        break; // Si una key funciona, ya tenemos la lista de modelos del proveedor
                    }
                }
            }
        }
    }

    // 2. Vertex AI (GCloud Integration)
    if let Some(project) = &keys.gcloud_project_id {
        // Ignorar proyectos de AI Studio que no son compatibles con Vertex AI SDK estándar
        if !project.is_empty() && !project.starts_with("gen-lang-client-") {
            let region = keys
                .gcloud_region
                .clone()
                .unwrap_or_else(|| "us-central1".to_string());

            // Intentar obtener modelos de Vertex si hay auth
            let token_output = tokio::process::Command::new("gcloud")
                .arg("auth")
                .arg("print-access-token")
                .output()
                .await;

            if let Ok(out) = token_output {
                if out.status.success() {
                    let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    let url = format!("https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models", 
                        region, project, region);

                    let res = client
                        .get(url)
                        .header("Authorization", format!("Bearer {}", token))
                        .send()
                        .await;

                    if let Ok(resp) = res {
                        if resp.status().is_success() {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(v_models) = json["models"].as_array() {
                                    for m in v_models {
                                        if let Some(name) = m["name"].as_str() {
                                            if let Some(short_name) = name.split('/').last() {
                                                models.push(format!("vertex-{}", short_name));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modelos estáticos de Vertex como fallback de descubrimiento si la API falla pero el proyecto existe
            if !models.iter().any(|m| m.starts_with("vertex-")) {
                models.push("vertex-gemini-2.5-flash".into());
                models.push("gemini-3-pro-preview".into());
            }
        }
    }

    // 3. Perplexity
    if let Some(_api_key) = &keys.perplexity_api_key {
        // Perplexity no tiene endpoint de lista de modelos público/estándar fiable, usamos los actuales
        models.push("sonar-reasoning-pro".to_string());
        models.push("sonar-pro".to_string());
        models.push("sonar".to_string());
    }

    // 4. Groq
    if let Some(api_key) = &keys.groq_api_key {
        let res = client
            .get("https://api.groq.com/openai/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await;

        if let Ok(resp) = res {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(data) = json["data"].as_array() {
                        for m in data {
                            if let Some(id) = m["id"].as_str() {
                                models.push(id.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 5. OpenAI
    if let Some(api_key) = &keys.openai_api_key {
        let res = client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await;

        if let Ok(resp) = res {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(data) = json["data"].as_array() {
                        for m in data {
                            if let Some(id) = m["id"].as_str() {
                                if id.starts_with("gpt-") {
                                    models.push(id.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Limpieza: eliminar duplicados y ordenar
    models.sort();
    models.dedup();

    // Requisito de Resiliencia: Si no hay modelos detectados, NO morimos.
    // Ofrecemos los modelos estándar como fallback manual para que la GUI no se bloquee.
    if models.is_empty() {
        models.push("gemini-2.5-flash".into());
        models.push("gemini-3-pro-preview".into());
    }

    Ok(models)
}

#[tauri::command]
async fn escanear_sentinel_media_fabrica() -> Result<String, String> {
    let output = tokio::process::Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"))
        .arg("scan") // El nuevo CLI usa 'scan' directamente
        .current_dir(AGENTS_PATH)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn ejecutar_generacion_fabrica(
    window: tauri::Window,
    config: FactoryConfig,
) -> Result<String, String> {
    // Emitir evento de inicio
    let _ = window.emit("generacion-fabrica-iniciada", ());

    tokio::spawn(async move {
        let mut cmd = tokio::process::Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"));
        cmd.arg("factory"); // El subcomando sigue siendo 'factory' en el nuevo CLI

        // Mapear proveedor al flag correspondiente del nuevo CLI
        if config.provider == "gemini" {
            // El nuevo CLI usa Gemini por defecto si no se especifica otro
        } else {
            cmd.arg(format!("--{}", config.provider));
        }

        // Pasar flags de fabrica (segun config)
        if config.shorts {
            cmd.arg("--shorts");
        }
        if config.longform {
            cmd.arg("--longform");
        }
        if config.stitch {
            cmd.arg("--stitch");
        }
        if config.publish {
            cmd.arg("--publish");
        }
        if config.local {
            cmd.arg("--local");
        }
        if config.cinematic {
            cmd.arg("--remotion-render");
        }
        }
        if let Some(file) = config.specific_file {
            cmd.arg("--file");
            cmd.arg(file);
        }

        match cmd.current_dir(AGENTS_PATH).output().await {
            Ok(output) => {
                let success = output.status.success();
                let message = if success {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                } else {
                    String::from_utf8_lossy(&output.stderr).trim().to_string()
                };

                // Emitir evento de finalizacion
                let _ = window.emit(
                    "tarea-fabrica-completada",
                    FactoryTaskPayload { success, message },
                );

                // Si fue exitoso, la sentinel_media pudo cambiar (videos/md nuevos), disparar update
                if success {
                    let _ = window.emit("indice-sentinel_media-actualizado", ());
                }
            },
            Err(e) => {
                let _ = window.emit(
                    "tarea-fabrica-completada",
                    FactoryTaskPayload {
                        success: false,
                        message: format!("Error al lanzar proceso: {}", e),
                    },
                );
            },
        }
    });

    Ok("Generacion de fabrica iniciada en segundo plano...".to_string())
}

#[tauri::command]
async fn refrescar_indice_sentinel_media() -> Result<String, String> {
    let output = tokio::process::Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"))
        .arg("scan") // Usar el comando 'scan' consolidado
        .current_dir(AGENTS_PATH)
        .output()
        .await
        .map_err(|e| format!("Error ejecutando sentinel scan: {}", e))?;

    if output.status.success() {
        Ok("Índice de la biblioteca actualizado correctamente.".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[tauri::command]
async fn get_crystal_resonance() -> Result<Vec<f32>, String> {
    let crystal_path = Path::new("/var/lib/pai60/memory.crystal");
    if !crystal_path.exists() {
        return Ok(vec![0.0; 12]); // Fallback si no hay Ring 0 activo
    }

    let store = CrystalStore::open(crystal_path, 12)
        .map_err(|e| format!("Error abriendo Crystal Store: {}", e))?;
    
    let lattice = store.load();
    let amplitudes: Vec<f32> = lattice.crystals
        .iter()
        .map(|c| c.amplitude.to_raw() as f32 / 1_000_000.0) // Aproximación decimal para la GUI
        .collect();

    Ok(amplitudes)
}

// ============================================================================
// GESTIÓN DE CONVERSACIONES
// ============================================================================

fn get_conversations_dir() -> PathBuf {
    PathBuf::from(AGENTS_PATH).join(".gemini/conversations")
}

#[tauri::command]
fn guardar_conversacion(name: String, messages: Vec<ChatMessage>) -> Result<String, String> {
    let conv_dir = get_conversations_dir();
    fs::create_dir_all(&conv_dir).map_err(|e| format!("Error creando directorio: {}", e))?;

    let history = ChatHistory { messages };
    let filename = format!("{}.json", name.replace(" ", "_"));
    let filepath = conv_dir.join(filename);

    let json = serde_json::to_string_pretty(&history)
        .map_err(|e| format!("Error serializando conversación: {}", e))?;

    fs::write(&filepath, json).map_err(|e| format!("Error guardando conversación: {}", e))?;

    Ok(format!("Conversación '{}' guardada correctamente", name))
}

#[tauri::command]
fn listar_conversaciones() -> Result<Vec<String>, String> {
    let conv_dir = get_conversations_dir();

    if !conv_dir.exists() {
        return Ok(Vec::new());
    }

    let entries =
        fs::read_dir(&conv_dir).map_err(|e| format!("Error leyendo directorio: {}", e))?;

    let mut conversations = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    conversations.push(name.trim_end_matches(".json").replace("_", " "));
                }
            }
        }
    }

    conversations.sort();
    Ok(conversations)
}

#[tauri::command]
fn cargar_conversacion(name: String) -> Result<Vec<ChatMessage>, String> {
    let conv_dir = get_conversations_dir();
    let filename = format!("{}.json", name.replace(" ", "_"));
    let filepath = conv_dir.join(filename);

    if !filepath.exists() {
        return Err(format!("Conversación '{}' no encontrada", name));
    }

    let content =
        fs::read_to_string(&filepath).map_err(|e| format!("Error leyendo conversación: {}", e))?;

    let history: ChatHistory = serde_json::from_str(&content)
        .map_err(|e| format!("Error parseando conversación: {}", e))?;

    Ok(history.messages)
}

#[tauri::command]
fn eliminar_conversacion(name: String) -> Result<String, String> {
    let conv_dir = get_conversations_dir();
    let filename = format!("{}.json", name.replace(" ", "_"));
    let filepath = conv_dir.join(filename);

    if !filepath.exists() {
        return Err(format!("Conversación '{}' no encontrada", name));
    }

    fs::remove_file(&filepath).map_err(|e| format!("Error eliminando conversación: {}", e))?;

    Ok(format!("Conversación '{}' eliminada", name))
}

#[tauri::command]
fn exportar_conversacion_md(name: String) -> Result<String, String> {
    let messages = cargar_conversacion(name.clone())?;

    let mut markdown = format!("# Conversación: {}\n\n", name);
    markdown.push_str(&format!(
        "**Fecha:** {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    markdown.push_str("---\n\n");

    for msg in messages {
        let role_label = if msg.role == "user" {
            "👤 Usuario"
        } else {
            "🤖 Asistente"
        };
        markdown.push_str(&format!("## {} ({})\n\n", role_label, msg.timestamp));
        markdown.push_str(&msg.content);
        markdown.push_str("\n\n---\n\n");
    }

    let export_path =
        PathBuf::from(AGENTS_PATH).join(format!("conversacion_{}.md", name.replace(" ", "_")));

    fs::write(&export_path, markdown)
        .map_err(|e| format!("Error exportando conversación: {}", e))?;

    Ok(format!(
        "Conversación exportada a: {}",
        export_path.display()
    ))
}

// ============================================================================
// PROCESAMIENTO DE ARCHIVOS
// ============================================================================

#[tauri::command]
async fn analizar_archivo(file_path: String) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"))
        .arg("research")
        .arg("--file")
        .arg(&file_path)
        .arg("--deep")
        .current_dir(AGENTS_PATH)
        .output()
        .map_err(|e| format!("Error ejecutando sentinel research: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Error analizando archivo: {}", stderr))
    }
}

#[tauri::command]
async fn traducir_archivo(file_path: String, target_lang: String) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"))
        .arg("research")
        .arg("--file")
        .arg(&file_path)
        .arg("--translate")
        .arg("--target-lang")
        .arg(&target_lang)
        .current_dir(AGENTS_PATH)
        .output()
        .map_err(|e| format!("Error ejecutando sentinel research: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Extraer la ruta del archivo traducido del output
        if let Some(line) = stdout
            .lines()
            .find(|l| l.contains("Traducción guardada en:"))
        {
            Ok(line
                .replace("✅ Traducción guardada en:", "")
                .trim()
                .to_string())
        } else {
            Ok(stdout.to_string())
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Error traduciendo archivo: {}", stderr))
    }
}

// Rastreador global de tareas activas
lazy_static::lazy_static! {
    static ref ACTIVE_RESEARCH_TASKS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
}

#[tauri::command]
async fn iniciar_tarea_investigacion(
    window: tauri::Window,
    query: String,
    mode: String,
    grounding: bool,
) -> Result<String, String> {
    let mut cmd = tokio::process::Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"));

    // CRITICO: fijar directorio actual en AGENTS_PATH para encontrar 'sentinel_keys.json'
    cmd.current_dir(AGENTS_PATH);

    cmd.arg("research");

    // Agregar a tareas activas
    if let Ok(mut tasks) = ACTIVE_RESEARCH_TASKS.lock() {
        tasks.push(format!("[{}] {}", mode, query));
    }

    match mode.as_str() {
        "ARCHITECT" => {
            cmd.arg("--system");
            cmd.arg("--prompt").arg(format!("MODO ARQUITECTO DE SISTEMAS: Analiza la siguiente consulta desde una perspectiva de arquitectura, escalabilidad y patrones de diseño: {}", query));
        },
        "LINGUIST" => {
            cmd.arg("--translate");
            cmd.arg("--prompt").arg(format!(
                "MODO DETECTIVE LINGÜÍSTICO: Analiza semántica, etimología y discurso de: {}",
                query
            ));
        },
        "PATTERN" => {
            cmd.arg("--imagina");
            cmd.arg("--prompt").arg(format!("MODO COMPARADOR DE PATRONES: Busca conexiones cruzadas y polinización entre dominios para: {}", query));
        },
        "GENEALOGIST" => {
            cmd.arg("--deep");
            cmd.arg("--prompt").arg(format!(
                "MODO GENEALOGISTA: Rastrea el origen, evolución y argumentos históricos de: {}",
                query
            ));
        },
        "DEEP_DIVE" | _ => {
            cmd.arg("--deep");
            cmd.arg("--prompt").arg(&query);
        },
    }

    if grounding {
        cmd.arg("--telos-context");
    }

    let mode_clone = mode.clone();
    let query_clone = query.clone();

    tokio::spawn(async move {
        match cmd.output().await {
            Ok(output) => {
                // Remover de tareas activas
                if let Ok(mut tasks) = ACTIVE_RESEARCH_TASKS.lock() {
                    tasks.retain(|t| t != &format!("[{}] {}", mode_clone, query_clone));
                }

                let success = output.status.success();
                if success {
                    eprintln!("Tarea de investigacion [{}] completada", mode_clone);
                    let _ = window.emit("reportes-investigacion-actualizados", ());
                } else {
                    eprintln!(
                        "Tarea de investigacion [{}] fallo: {}",
                        mode_clone,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }

                let _ = window.emit(
                    "tarea-investigacion-completada",
                    ResearchTaskPayload {
                        mode: mode_clone,
                        query: query_clone,
                        success,
                        message: None,
                    },
                );
            },
            Err(e) => {
                // Remover de tareas activas tambien en error
                if let Ok(mut tasks) = ACTIVE_RESEARCH_TASKS.lock() {
                    tasks.retain(|t| t != &format!("[{}] {}", mode_clone, query_clone));
                }
                eprintln!("No se pudo lanzar la tarea de investigacion: {}", e);
                let _ = window.emit(
                    "tarea-investigacion-completada",
                    ResearchTaskPayload {
                        mode: mode_clone,
                        query: query_clone,
                        success: false,
                        message: Some(e.to_string()),
                    },
                );
            },
        }
    });

    Ok(format!(
        "Protocolo de investigación {} iniciado para: {}",
        mode, query
    ))
}

#[tauri::command]
fn get_tareas_investigacion_activas() -> Vec<String> {
    if let Ok(tasks) = ACTIVE_RESEARCH_TASKS.lock() {
        tasks.clone()
    } else {
        Vec::new()
    }
}

#[tauri::command]
async fn execute_sentinel_command(command: String) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"))
        .args(command.split_whitespace())
        .current_dir(AGENTS_PATH)
        .output()
        .map_err(|e| format!("Error ejecutando comando: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!(
            "Error (Code {}):\nStdout: {}\nStderr: {}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        ))
    }
}

#[tauri::command]
async fn execute_gemini_command(command: String) -> Result<String, String> {
    use std::process::Command;

    let args: Vec<&str> = command.split_whitespace().collect();
    if args.is_empty() {
        return Err("Comando vacío".into());
    }

    let output = Command::new("gemini")
        .args(&args)
        .current_dir(AGENTS_PATH)
        .output()
        .map_err(|e| format!("Error ejecutando gemini: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!(
            "Error (Code {}):\nStdout: {}\nStderr: {}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        ))
    }
}

#[tauri::command]
async fn ingestar_memoria(path: String) -> Result<String, String> {
    let output = Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"))
        .arg("memory")
        .arg("ingest")
        .arg("--path")
        .arg(&path)
        .current_dir(AGENTS_PATH)
        .output()
        .map_err(|e| format!("Error en ingestión: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn consultar_memoria(query: String) -> Result<String, String> {
    let output = Command::new(PathBuf::from(AGENTS_PATH).join("sentinel"))
        .arg("memory")
        .arg("query")
        .arg(&query)
        .current_dir(AGENTS_PATH)
        .output()
        .map_err(|e| format!("Error en consulta de memoria: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// ============================================================================
// API COST & USAGE MANAGEMENT COMMANDS
// ============================================================================

#[tauri::command]
fn get_detalles_uso_sentinomica() -> UsoGlobalSentinomica {
    let path = PathBuf::from(AGENTS_PATH).join("sentinel_usage.json");
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        UsoGlobalSentinomica::default()
    }
}

#[tauri::command]
fn get_detalles_proveedor(provider: String) -> Result<SentinomicaStats, String> {
    let usage = get_detalles_uso_sentinomica();
    usage
        .provider_stats
        .get(&provider)
        .cloned()
        .ok_or_else(|| format!("Provider '{}' not found", provider))
}

#[tauri::command]
fn get_llamadas_api_recientes(provider: Option<String>, limit: Option<usize>) -> Vec<ApiCall> {
    let usage = get_detalles_uso_sentinomica();
    let mut calls = usage.recent_calls;

    if let Some(p) = provider {
        calls.retain(|c| c.provider == p);
    }

    let limit = limit.unwrap_or(20).min(100);
    calls.into_iter().rev().take(limit).collect()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CostSummary {
    pub total_today: f32,
    pub total_this_month: f32,
    pub total_all_time: f32,
    pub by_provider: HashMap<String, ProviderCostBreakdown>,
    pub daily_budget: f32,
    pub monthly_budget: f32,
    pub budget_alert_threshold: f32,
    pub is_over_daily_budget: bool,
    pub is_over_monthly_budget: bool,
    pub daily_budget_usage_pct: f32,
    pub monthly_budget_usage_pct: f32,
    pub total_revenue_today: f32,
    pub total_revenue_this_month: f32,
    pub total_revenue_all_time: f32,
    pub global_roi_index: f32,
    pub active_assets_count: u32,
    pub smart_advice: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderCostBreakdown {
    pub today: f32,
    pub this_month: f32,
    pub all_time: f32,
    pub requests_today: u32,
    pub avg_cost_per_request: f32,
    pub avg_efficiency_score: f32,
    pub hardware_overhead_factor: f32,
}

#[tauri::command]
fn get_resumen_costos() -> CostSummary {
    let usage = get_detalles_uso_sentinomica();

    let mut total_today = 0.0;
    let mut total_this_month = 0.0;
    let mut total_all_time = 0.0;
    let mut by_provider = HashMap::new();

    let mut total_revenue_today = 0.0;
    let mut total_revenue_this_month = 0.0;
    let mut total_revenue_all_time = 0.0;

    for (provider, stats) in &usage.provider_stats {
        total_today += stats.cost_today_usd;
        total_this_month += stats.cost_this_month_usd;
        total_all_time += stats.total_cost_usd;

        total_revenue_today += stats.revenue_today_usd;
        total_revenue_this_month += stats.revenue_this_month_usd;
        total_revenue_all_time += stats.total_revenue_usd;

        by_provider.insert(
            provider.clone(),
            ProviderCostBreakdown {
                today: stats.cost_today_usd,
                this_month: stats.cost_this_month_usd,
                all_time: stats.total_cost_usd,
                requests_today: stats.requests,
                avg_cost_per_request: if stats.requests > 0 {
                    stats.total_cost_usd / stats.requests as f32
                } else {
                    0.0
                },
                avg_efficiency_score: stats.avg_efficiency_score,
                hardware_overhead_factor: stats.hardware_overhead_factor,
            },
        );
    }

    let daily_budget = usage.daily_budget_usd;
    let monthly_budget = usage.monthly_budget_usd;
    let daily_budget_usage_pct = if daily_budget > 0.0 {
        (total_today / daily_budget) * 100.0
    } else {
        0.0
    };
    let monthly_budget_usage_pct = if monthly_budget > 0.0 {
        (total_this_month / monthly_budget) * 100.0
    } else {
        0.0
    };

    let global_roi_index = if total_all_time > 0.0 {
        total_revenue_all_time / total_all_time
    } else {
        0.0
    };

    // Count active assets (recursively count .md files in Vault)
    let active_assets_count = count_markdown_files(PathBuf::from(VAULT_PATH));

    // Generate dynamic advice based on usage/ROI
    let mut smart_advice = Vec::new();
    if global_roi_index > 2.0 {
        smart_advice.push(
            "ROI EXCELENTE: Considera aumentar el presupuesto de producción para escalar activos."
                .to_string(),
        );
    } else if global_roi_index > 0.0 {
        smart_advice.push(
            "ROI ESTABLE: Optimiza los prompts de los agentes para reducir latencia y costo."
                .to_string(),
        );
    } else {
        smart_advice.push(
            "PENDIENTE: Genera más contenido para empezar a trackear el ROI global.".to_string(),
        );
    }

    if total_today > daily_budget && daily_budget > 0.0 {
        smart_advice.push(
            "ALERTA DE BUDGET: Has excedido el presupuesto diario. Revisa los modelos en uso."
                .to_string(),
        );
    }

    if let Some((best_provider, _)) = by_provider.iter().max_by(|a, b| {
        a.1.avg_efficiency_score
            .partial_cmp(&b.1.avg_efficiency_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        smart_advice.push(format!(
            "RECOMENDADO: El proveedor {} está mostrando la mejor eficiencia hoy.",
            best_provider.to_string().to_uppercase()
        ));
    }

    CostSummary {
        total_today,
        total_this_month,
        total_all_time,
        by_provider,
        daily_budget,
        monthly_budget,
        budget_alert_threshold: usage.alert_threshold_pct,
        is_over_daily_budget: daily_budget > 0.0 && total_today > daily_budget,
        is_over_monthly_budget: monthly_budget > 0.0 && total_this_month > monthly_budget,
        daily_budget_usage_pct,
        monthly_budget_usage_pct,
        total_revenue_today,
        total_revenue_this_month,
        total_revenue_all_time,
        global_roi_index,
        active_assets_count,
        smart_advice,
    }
}

fn count_markdown_files(dir: PathBuf) -> u32 {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_markdown_files(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                count += 1;
            }
        }
    }
    count
}

#[tauri::command]
fn establecer_presupuesto(daily: f64, monthly: f64, threshold: f64) -> Result<String, String> {
    let path = PathBuf::from(AGENTS_PATH).join("sentinel_usage.json");
    let mut usage: UsoGlobalSentinomica = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        UsoGlobalSentinomica::default()
    };

    usage.daily_budget_usd = daily as f32;
    usage.monthly_budget_usd = monthly as f32;
    usage.alert_threshold_pct = threshold as f32;

    if let Ok(json) = serde_json::to_string_pretty(&usage) {
        fs::write(path, json).map_err(|e| format!("Error salvando presupuesto: {}", e))?;
        Ok(format!(
            "Presupuesto establecido: ${}/día, ${}/mes, {}% umbral de alerta",
            daily, monthly, threshold
        ))
    } else {
        Err("Error serializando presupuesto".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CostProjection {
    pub current_daily_avg: f32,
    pub projected_month_end: f32,
    pub days_until_budget_exceeded: Option<u32>,
    pub recommended_daily_limit: f32,
}

#[tauri::command]
fn get_cost_projection() -> CostProjection {
    let usage = get_detalles_uso_sentinomica();
    let summary = get_resumen_costos();

    // Calculate average daily cost from recent calls
    let days_with_data: std::collections::HashSet<_> = usage
        .recent_calls
        .iter()
        .filter_map(|c| chrono::DateTime::parse_from_rfc3339(&c.timestamp).ok())
        .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.date_naive())
        .collect();

    let current_daily_avg = if !days_with_data.is_empty() {
        summary.total_this_month / days_with_data.len() as f32
    } else {
        0.0
    };

    // Get days in current month
    let now = chrono::Local::now().date_naive();
    let year = now.year();
    let month = now.month();
    let days_in_month = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .and_then(|next_month| next_month.pred_opt())
    .map(|last_day| last_day.day())
    .unwrap_or(30);

    let projected_month_end = current_daily_avg * days_in_month as f32;

    let days_until_budget_exceeded = if usage.monthly_budget_usd > 0.0 && current_daily_avg > 0.0 {
        let remaining_budget = usage.monthly_budget_usd - summary.total_this_month;
        if remaining_budget > 0.0 {
            Some((remaining_budget / current_daily_avg) as u32)
        } else {
            Some(0)
        }
    } else {
        None
    };

    let recommended_daily_limit = if usage.monthly_budget_usd > 0.0 {
        usage.monthly_budget_usd / days_in_month as f32
    } else {
        0.0
    };

    CostProjection {
        current_daily_avg,
        projected_month_end,
        days_until_budget_exceeded,
        recommended_daily_limit,
    }
}

// ============================================================================
// SYSTEM PROMPT / COGNITIVE LAYER MANAGEMENT
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemPromptInfo {
    pub name: String,
    pub filename: String,
    pub path: String,
}

#[tauri::command]
fn get_prompts_sistema() -> Vec<SystemPromptInfo> {
    let dir = PathBuf::from(AGENTS_PATH).join("prompts");
    let mut prompts = Vec::new();

    // Core Prompts que deben aparecer primero
    let core_prompts = vec![
        ("SENTINEL_PROMPT.md", "SENTINEL PRIME"),
        ("ANTIGRAVITY_CONTEXT.md", "ANTIGRAVITY CORE"),
    ];

    for (file, name) in core_prompts {
        let p = dir.join(file);
        if p.exists() {
            prompts.push(SystemPromptInfo {
                name: name.to_string(),
                filename: file.to_string(),
                path: p.to_string_lossy().to_string(),
            });
        }
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    // Evitar duplicar los core prompts y archivos irrelevantes
                    if filename == "SENTINEL_PROMPT.md"
                        || filename == "ANTIGRAVITY_CONTEXT.md"
                        || filename.starts_with('.')
                    {
                        continue;
                    }

                    // Incluir todos los archivos .md de prompts/ (excepto index.md)
                    if filename.ends_with(".md") && filename != "index.md" {
                        let name = filename
                            .strip_suffix(".md")
                            .unwrap_or(filename)
                            .replace('_', " ")
                            .to_uppercase();

                        // Verificar tamaño > 0 antes de añadir
                        if let Ok(meta) = path.metadata() {
                            if meta.len() > 0 {
                                prompts.push(SystemPromptInfo {
                                    name,
                                    filename: filename.to_string(),
                                    path: path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    prompts
}

#[tauri::command]
fn leer_prompt_sistema(filename: String) -> Result<String, String> {
    let path = PathBuf::from(AGENTS_PATH).join(filename);
    if !path.exists() {
        return Err(format!(
            "Archivo de prompt '{}' no encontrado",
            path.display()
        ));
    }
    fs::read_to_string(path).map_err(|e| format!("Error leyendo prompt: {}", e))
}

#[tauri::command]
fn guardar_prompt_sistema(filename: String, content: String) -> Result<String, String> {
    let path = PathBuf::from(AGENTS_PATH).join(filename);
    fs::write(path, content).map_err(|e| format!("Error guardando prompt: {}", e))?;
    Ok("Prompt guardado correctamente".to_string())
}

#[tauri::command]
async fn check_gpu_status() -> HardwareStatus {
    use nvml_wrapper::Nvml;

    // Intentamos una lectura rápida (On-Demand)
    // El monitor en segundo plano es la fuente de verdad principal.
    // Esto es un fallback inicial.
    match Nvml::init() {
        Ok(nvml) => {
            if let Ok(device) = nvml.device_by_index(0) {
                let temp = device
                    .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                    .unwrap_or(0) as f32;
                let usage = device
                    .utilization_rates()
                    .map(|r| r.gpu as f32)
                    .unwrap_or(0.0);
                let memory = device
                    .memory_info()
                    .map(|m| format!("{}MiB / {}MiB", m.used / 1024 / 1024, m.total / 1024 / 1024))
                    .unwrap_or("?".to_string());

                return HardwareStatus::Active {
                    temp,
                    usage,
                    memory,
                    fan_speed: device.fan_speed(0).ok(),
                };
            }
        },
        Err(_) => {},
    }

    HardwareStatus::Offline {
        last_seen: chrono::Local::now().to_string(),
        error: Some("NVML Init Failed or GPU Busy".to_string()),
    }
}

const VAULT_PATH: &str = "vault";
const AGENTS_PATH: &str = ".";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlimptonRatio {
    pub row: u32,
    pub decimal_approx: f32,
    pub s60_repr: String,
    pub is_axion: bool,
}

#[tauri::command]
fn get_plimpton_ratios() -> Vec<PlimptonRatio> {
    vec![
        PlimptonRatio {
            row: 1,
            decimal_approx: 1.983,
            s60_repr: "1; 59, 0, 15".into(),
            is_axion: false,
        },
        PlimptonRatio {
            row: 12,
            decimal_approx: 1.534,
            s60_repr: "1; 32, 2, 24".into(),
            is_axion: true,
        },
        // ... extendible
    ]
}

#[tauri::command]
fn get_thermal_status() -> String {
    let thermal_zone_path = "/sys/class/thermal/thermal_zone0/temp";
    if let Ok(content) = fs::read_to_string(thermal_zone_path) {
        if let Ok(temp_millidegrees) = content.trim().parse::<f32>() {
            return format!("CPU: {:.1}°C", temp_millidegrees / 1000.0);
        }
    }
    "N/A".to_string()
}

// ============================================================================
// COMANDOS DE CONTROL DE FÁBRICA
// ============================================================================

/// Limpia el estado de la fábrica y verifica permisos
#[tauri::command]
fn cleanup_factory_state() -> Result<String, String> {
    let ops_path_buf = PathBuf::from(VAULT_PATH).join("SecurePenguin/.sentinel/operations.json");
    let ops_path = ops_path_buf.to_str().unwrap_or("");

    // Verificar permisos de escritura
    let sentinel_dir_buf = PathBuf::from(VAULT_PATH).join("SecurePenguin/.sentinel");
    let sentinel_dir = sentinel_dir_buf.as_path();
    if !sentinel_dir.exists() {
        fs::create_dir_all(sentinel_dir)
            .map_err(|e| format!("Error creando directorio .sentinel: {}", e))?;
    }

    // Limpiar operations.json si existe y es antiguo (>24h)
    if Path::new(ops_path).exists() {
        if let Ok(metadata) = fs::metadata(ops_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() > 86400 {
                        fs::remove_file(ops_path).map_err(|e| {
                            format!("Error eliminando operations.json antiguo: {}", e)
                        })?;
                        return Ok(
                            "Limpieza completada: operations.json antiguo eliminado".to_string()
                        );
                    }
                }
            }
        }
    }

    Ok("Sistema listo para ejecución".to_string())
}

/// Ejecuta el agente de fábrica en modo daemon
#[tauri::command]
async fn run_factory_agent(app_handle: tauri::AppHandle) -> Result<String, String> {
    // Verificar si ya hay un proceso corriendo
    {
        let mut process = FACTORY_PROCESS
            .lock()
            .map_err(|e| format!("Error accediendo al estado del proceso: {}", e))?;

        if let Some(ref mut child) = *process {
            // Verificar si el proceso sigue vivo
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Proceso terminó, limpiar
                    *process = None;
                },
                Ok(None) => {
                    return Err("El agente de fábrica ya está ejecutándose".to_string());
                },
                Err(e) => {
                    return Err(format!("Error verificando estado del proceso: {}", e));
                },
            }
        }
    }

    // Ejecutar el binario sentinel en modo daemon
    let sentinel_path = PathBuf::from(AGENTS_PATH).join("sentinel");

    let child = Command::new(&sentinel_path)
        .arg("daemon")
        .current_dir(AGENTS_PATH)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Error ejecutando sentinel: {}", e))?;

    let pid = child.id();

    // Guardar el proceso
    {
        let mut process = FACTORY_PROCESS
            .lock()
            .map_err(|e| format!("Error guardando proceso: {}", e))?;
        *process = Some(child);
    }

    // Emitir evento de inicio
    let _ = app_handle.emit("factory-started", pid);

    Ok(format!("Agente de fábrica iniciado (PID: {})", pid))
}

/// Detiene el agente de fábrica
#[tauri::command]
fn stop_factory_agent() -> Result<String, String> {
    let mut process = FACTORY_PROCESS
        .lock()
        .map_err(|e| format!("Error accediendo al estado del proceso: {}", e))?;

    if let Some(ref mut child) = *process {
        child
            .kill()
            .map_err(|e| format!("Error deteniendo el agente: {}", e))?;
        *process = None;
        Ok("Agente de fábrica detenido".to_string())
    } else {
        Err("No hay ningún agente ejecutándose".to_string())
    }
}

/// Obtiene el estado actual del agente de fábrica
#[tauri::command]
fn get_factory_status() -> Result<serde_json::Value, String> {
    let process = FACTORY_PROCESS
        .lock()
        .map_err(|e| format!("Error accediendo al estado del proceso: {}", e))?;

    if let Some(ref child) = *process {
        Ok(serde_json::json!({
            "running": true,
            "pid": child.id()
        }))
    } else {
        Ok(serde_json::json!({
            "running": false,
            "pid": null
        }))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Phase 4: Start Background Services
        .setup(|app| {
            let handle = app.handle().clone();
            // Iniciar Monitor GPU en segundo plano
            crate::services::gpu_monitor::start_monitor(handle.clone());
            // Iniciar Streamer de Logs
            crate::services::log_streamer::start_log_stream(handle.clone());
            // Iniciar Sincronización Redis Ring 0
            crate::redis_sync::start_redis_listener(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            iniciar_tarea_investigacion,
            get_plimpton_ratios,
            get_thermal_status,
            send_neural_message,
            obtener_modelos_disponibles,
            get_estadisticas_fabrica,
            consultar_memoria,
            get_estado_balanceador,
            get_detalles_uso_sentinomica,
            get_detalles_proveedor,
            get_llamadas_api_recientes,
            get_resumen_costos,
            establecer_presupuesto,
            get_cost_projection,
            check_gpu_status,
            get_llaves_api,
            guardar_llaves_api,
            obtener_uso_sentinomica,
            autenticar_gcloud,
            guardar_conversacion,
            listar_conversaciones,
            cargar_conversacion,
            eliminar_conversacion,
            get_operaciones,
            get_estadisticas_cortex,
            get_archivos_sentinel_media,
            get_reportes_investigacion,
            get_agentes,
            iniciar_agente,
            detener_agente,
            get_estado_agente_investigacion,
            get_tareas_investigacion_activas,
            get_logs_sistema,
            escanear_sentinel_media_fabrica,
            ingestar_memoria,
            ejecutar_generacion_fabrica,
            get_prompts_sistema,
            leer_prompt_sistema,
            guardar_prompt_sistema,
            leer_contenido_archivo_sentinel_media,
            execute_sentinel_command,
            execute_gemini_command,
            exportar_conversacion_md,
            analizar_archivo,
            traducir_archivo,
            refrescar_indice_sentinel_media,
            get_crystal_resonance,
            get_balancer_status,
            // Comandos de control de fábrica
            cleanup_factory_state,
            run_factory_agent,
            stop_factory_agent,
            get_factory_status
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar la aplicacion tauri");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_keys_existence() {
        let keys = load_keys();
        assert!(keys.gemini_api_keys.is_some());
    }

    #[test]
    fn test_path_resolutions() {
        let path = PathBuf::from(AGENTS_PATH).join("sentinel_keys.json");
        assert!(path.exists());
    }
}
