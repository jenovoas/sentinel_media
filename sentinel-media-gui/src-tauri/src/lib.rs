// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

const VAULT_PATH: &str = ".";
const AGENTS_PATH: &str = ".";

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
use sentinel_media_core::FactoryConfig;
use lazy_static::lazy_static;
use std::env;
use std::sync::Mutex;


pub mod factory;
pub mod services;

lazy_static! {
    static ref AGENTS_PATH: PathBuf = {
        if let Ok(path) = env::var("FACTORY_AGENTS_PATH") {
            PathBuf::from(path)
        } else {
            if let Ok(exe_path) = env::current_exe() {
                 if let Some(p) = exe_path.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).and_then(|p| p.parent()) {
                    return p.to_path_buf();
                }
            }
            PathBuf::from(".")
        }
    };
    static ref FACTORY_PROCESS: Mutex<Option<std::process::Child>> = Mutex::new(None);
}

#[tauri::command]
async fn get_active_gcp_project() -> Result<String, String> {
    let output = tokio::process::Command::new("gcloud")
        .arg("config")
        .arg("get-value")
        .arg("project")
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// ... (The rest of the file remains the same, so I will omit it for brevity, 
// but I will include the tauri builder part at the end)


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_active_gcp_project,
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
