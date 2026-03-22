use crate::HardwareStatus;
use log::{error, info};
use nvml_wrapper::Nvml;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

// Configuración del Monitor
const MONITOR_INTERVAL_MS: u64 = 2000; // Intervalo de ciclo mayor
const BURST_SAMPLES: usize = 5; // Muestras por ciclo para suavizado
const BURST_DELAY_MS: u64 = 50; // Delay entre muestras de ráfaga
const TEMP_DELTA_THRESHOLD: f32 = 1.0;
const USAGE_DELTA_THRESHOLD: f32 = 5.0;

struct GpuMetrics {
    temp: f32,
    usage: f32,
    memory_used: u64,
    memory_total: u64,
    fan_speed: Option<u32>,
}

/// Helper para leer una ráfaga de datos y promediarlos usando Iteradores Zero-Cost.
/// Propaga errores con `?` si el dispositivo se desconecta.
fn read_gpu_burst(
    device: &nvml_wrapper::Device,
) -> Result<GpuMetrics, nvml_wrapper::error::NvmlError> {
    let mut temps = Vec::with_capacity(BURST_SAMPLES);
    let mut usages = Vec::with_capacity(BURST_SAMPLES);

    // 1. Burst Acquisition
    for _ in 0..BURST_SAMPLES {
        temps.push(
            device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)? as f32,
        );
        usages.push(device.utilization_rates()?.gpu as f32);

        // Pequeña pausa para capturar micro-variaciones
        thread::sleep(Duration::from_millis(BURST_DELAY_MS));
    }

    // 2. Zero-Cost Abstraction: Signal Smoothing via Iterators
    // Filtramos valores espurios (<= 0) y promediamos.
    let avg_temp = temps.iter().filter(|&&t| t > 0.0).sum::<f32>() / (temps.len() as f32).max(1.0);

    let avg_usage = usages.iter().sum::<f32>() / (usages.len() as f32).max(1.0);

    // Memoria y Fan (Lectura única, suele ser estable)
    let memory = device.memory_info()?;
    let fan = device.fan_speed(0).ok();

    Ok(GpuMetrics {
        temp: avg_temp,
        usage: avg_usage.round(), // Redondear uso para UI
        memory_used: memory.used,
        memory_total: memory.total,
        fan_speed: fan,
    })
}

/// Inicia el monitor de GPU en un hilo dedicado.
/// Implementa gestión de errores robusta y "Signal Smoothing".
pub fn start_monitor(app: AppHandle) {
    thread::spawn(move || {
        info!("Iniciando GPU Monitor Thread (NVML) v2.0 - Zero Cost Abstractions");

        // Inicialización "Fallible" - Si falla aquí, el hilo muere reportando error.
        let nvml = match Nvml::init() {
            Ok(n) => n,
            Err(e) => {
                error!("CRITICAL: NVML Init Failed: {}", e);
                emit_error(&app, format!("NVML Init Failed: {}", e));
                return;
            },
        };

        // Bucle de re-conexión (Resilience Pattern)
        // Si perdemos la GPU, intentamos recuperarla en el siguiente ciclo en lugar de panic.
        loop {
            // Intentar obtener handle del dispositivo
            let device = match nvml.device_by_index(0) {
                Ok(d) => d,
                Err(e) => {
                    error!("GPU no encontrada o perdida: {}", e);
                    emit_error(&app, "GPU Detached / Offline".into());
                    thread::sleep(Duration::from_secs(5));
                    continue;
                },
            };

            // Estado local para Deltas
            let mut last_processed = GpuMetrics {
                temp: 0.0,
                usage: 0.0,
                memory_used: 0,
                memory_total: 0,
                fan_speed: None,
            };

            // Inner Loop de Monitoreo Activo
            loop {
                match read_gpu_burst(&device) {
                    Ok(metrics) => {
                        // Lógica de "Quiet Mode" con las métricas suavizadas
                        let temp_diff = (metrics.temp - last_processed.temp).abs();
                        let usage_diff = (metrics.usage - last_processed.usage).abs();

                        if temp_diff > TEMP_DELTA_THRESHOLD || usage_diff > USAGE_DELTA_THRESHOLD {
                            let status = HardwareStatus::Active {
                                temp: metrics.temp,
                                usage: metrics.usage,
                                memory: format!(
                                    "{}MiB / {}MiB",
                                    metrics.memory_used / 1024 / 1024,
                                    metrics.memory_total / 1024 / 1024
                                ),
                                fan_speed: metrics.fan_speed,
                            };

                            if let Err(e) = app.emit("gpu-metrics", &status) {
                                error!("Frontend desconectado?: {}", e);
                            }

                            last_processed = metrics;
                        }
                    },
                    Err(e) => {
                        // Aquí atrapamos la desconexión del sensor gracias al operador ?
                        error!("Error leyendo sensor GPU: {}", e);
                        emit_error(&app, format!("Sensor Error: {}", e));
                        break; // Salir al loop externo para re-intentar hook del dispositivo
                    },
                }

                thread::sleep(Duration::from_millis(MONITOR_INTERVAL_MS));
            }
        }
    });
}

fn emit_error(app: &AppHandle, msg: String) {
    let _ = app.emit(
        "gpu-metrics",
        &HardwareStatus::Offline {
            last_seen: chrono::Local::now().to_string(),
            error: Some(msg),
        },
    );
}
