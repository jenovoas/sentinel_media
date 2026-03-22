use log::{error, info};
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, Debug)]
pub enum LogSeverity {
    Info,
    Warning,
    Critical,
    HardwareAlert,
}

#[derive(Serialize, Clone, Debug)]
pub struct ProcessedLog {
    pub message: String,
    pub severity: LogSeverity,
    pub timestamp: u64,
}

/// Inicia el streaming de logs en un hilo dedicado.
/// Ejecuta `journalctl -f` y filtra eventos relevantes antes de emitir.
pub fn start_log_stream(app: AppHandle) {
    thread::spawn(move || {
        info!("Iniciando Log Streamer Thread (journalctl) con Smart Filtering...");

        let mut child = match Command::new("journalctl")
            .args(&["-f", "--output=cat", "--no-pager"]) // output=cat para mensaje crudo, timestamp lo ponemos nosotros
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                error!("Fallo al iniciar journalctl stream: {}", e);
                return;
            },
        };

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                match line {
                    Ok(raw_line) => {
                        // Lógica de filtrado inteligente ("Smart Filtering")
                        // Usamos match o ifs para categorizar
                        let upper = raw_line.to_uppercase();

                        let severity = if upper.contains("ERROR")
                            || upper.contains("FATAL")
                            || upper.contains("PANIC")
                        {
                            LogSeverity::Critical
                        } else if upper.contains("WARN") {
                            LogSeverity::Warning
                        } else if (upper.contains("GPU")
                            || upper.contains("TEMP")
                            || upper.contains("THERMAL"))
                            && upper.contains("CRITICAL")
                        {
                            LogSeverity::HardwareAlert
                        } else if upper.contains("SENTINEL") || upper.contains("CORTEX") {
                            // Logs operativos propios del sistema
                            LogSeverity::Info
                        } else {
                            // Ignoramos ruido de sistema irrelevante
                            continue;
                        };

                        let processed = ProcessedLog {
                            message: raw_line.clone(),
                            severity,
                            timestamp: SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        };

                        // Emitir evento estructurado
                        if let Err(e) = app.emit("sys-log", &processed) {
                            error!("Error emitiendo sys-log: {}", e);
                        }
                    },
                    Err(e) => error!("Error leyendo linea de journalctl: {}", e),
                }
            }
        }
    });
}
