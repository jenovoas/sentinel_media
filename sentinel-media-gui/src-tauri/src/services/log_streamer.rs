// src/services/log_streamer.rs
use log::{error, info};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter};

use crate::{LogSeverity, ProcessedLog};

/// Inicia el streaming de logs en un hilo dedicado.
/// Ejecuta `journalctl -f` y filtra eventos relevantes antes de emitir.
pub fn start_log_stream(app: AppHandle) {
    thread::spawn(move || {
        info!("Iniciando Log Streamer Thread (journalctl) con Smart Filtering...");

        let mut child = match Command::new("journalctl")
            .args(&["-f", "--output=cat", "--no-pager"])
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
                            LogSeverity::Info
                        } else {
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
