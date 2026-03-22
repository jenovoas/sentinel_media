/*
// src/control_agent.rs
//! 🎛️ CONTROL ROOM AGENT 🎛️
//! Monitorea el archivo Markdown y coordina las señales internas.
//! AHORA CON SUPERPODERES: Parsing de comandos Regex y Live Logging.
//! v2.1: Visual Dashboard & Progress Tracking.

use me60os_core::agent_manager::AgentSPA;
use me60os_core::cortex::CortexEngine;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ControlState {
    pub helios_enabled: bool,
    pub helios_storm: bool,
    pub factory_queue: bool,
    pub factory_clean: bool,
    pub adm_viz: bool,
    pub adm_ping: bool,
}

#[derive(Clone, Debug)]
struct AsyncTask {
    name: String,
    start_time: SystemTime,
    estimated_duration: Duration,
}

pub struct ControlAgent {
    name: String,
    md_path: String,
    json_path: String,
    last_read: SystemTime,
    last_dashboard_update: SystemTime,
    state: ControlState,
    command_regex: Regex,
    active_tasks: Vec<AsyncTask>,
}

impl ControlAgent {
    pub fn new(name: &str, md_path: &str, json_path: &str) -> Self {
        Self {
            name: name.to_string(),
            md_path: md_path.to_string(),
            json_path: json_path.to_string(),
            last_read: SystemTime::UNIX_EPOCH,
            last_dashboard_update: SystemTime::UNIX_EPOCH,
            state: ControlState::default(),
            command_regex: Regex::new(r"(?i)-\s*\[x\]\s*\*\*([^\*]+)\*\*:\s*([^\n]+)").unwrap(),
            active_tasks: Vec::new(),
        }
    }

    fn parse_markdown(&mut self, content: &str) -> ControlState {
        let mut state = ControlState::default();
        let mut commands_found = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let lower = line.to_lowercase();
            let is_checked =
                lower.contains("[x]") || lower.contains("[ x]") || lower.contains("[x ]");

            if is_checked {
                // 1. Estados Booleanos (Legacy)
                if lower.contains("resonancia solar") {
                    state.helios_enabled = true;
                }
                if lower.contains("modo tormenta") {
                    state.helios_storm = true;
                }
                if lower.contains("visualizar red") {
                    state.adm_viz = true;
                }

                // 2. Comandos Avanzados (Regex)
                if let Some(caps) = self.command_regex.captures(line) {
                    let cmd = caps.get(1).map_or("", |m| m.as_str()).trim();
                    let param = caps
                        .get(2)
                        .map_or("", |m| m.as_str())
                        .trim()
                        .trim_matches('"');

                    self.log(&format!("⚡ Detected Command: {} -> {}", cmd, param));
                    commands_found.push((
                        line_idx,
                        line.to_string(),
                        cmd.to_string(),
                        param.to_string(),
                    ));
                }
            }
        }

        // Ejecutar comandos encontrados y limpiar checkbox
        for (_idx, original_line, cmd, param) in commands_found {
            self.execute_command(&cmd, &param);
            self.mark_done(&original_line);
        }

        state
    }

    fn execute_command(&mut self, cmd: &str, param: &str) {
        let cmd_lower = cmd.to_lowercase();
        let param_owned = param.to_string();

        let est_duration = if cmd_lower.contains("research") {
            Duration::from_secs(60)
        } else if cmd_lower.contains("produce") {
            Duration::from_secs(120)
        } else {
            Duration::from_secs(10)
        };

        // Track task for visual dashboard
        self.active_tasks.push(AsyncTask {
            name: format!("{}: {}", cmd, param),
            start_time: SystemTime::now(),
            estimated_duration: est_duration,
        });

        if cmd_lower.contains("research") {
            self.log(&format!("🔬 Starting Research on: {}", param));
            thread::spawn(move || {
                let _ = Command::new(resolve_sentinel_bin())
                    .arg("research")
                    .arg("--file")
                    .arg(&param_owned)
                    .output();
            });
        } else if cmd_lower.contains("produce") {
            self.log(&format!("🎥 Queueing Production: {}", param));
            thread::spawn(move || {
                let _ = Command::new(resolve_sentinel_bin())
                    .arg("produce")
                    .arg("--file")
                    .arg(&param_owned)
                    .arg("--video")
                    .spawn();
            });
        } else {
            self.log(&format!("❓ Unknown Command: {}", cmd));
        }
    }

    fn update_dashboard(&mut self) {
        // Remove completed tasks from tracking (simple heuristic for now)
        self.active_tasks.retain(|t| {
            t.start_time.elapsed().unwrap_or(Duration::ZERO)
                < t.estimated_duration + Duration::from_secs(5)
        });

        let mut dashboard = String::new();
        dashboard.push_str("\n> **SYSTEM STATUS**: 🟢 ONLINE | 41Hz\n");

        if self.active_tasks.is_empty() {
            dashboard.push_str("> **ACTIVE TASKS**: 💤 Idle\n");
        } else {
            dashboard.push_str("> **ACTIVE TASKS**:\n");
            for task in &self.active_tasks {
                let elapsed = task
                    .start_time
                    .elapsed()
                    .unwrap_or(Duration::ZERO)
                    .as_secs_f64();
                let total = task.estimated_duration.as_secs_f64();
                let progress = (elapsed / total).min(1.0);
                let percent = (progress * 100.0) as u32;

                // Generate ASCII Bar: [████░░░░░░]
                let bar_len = 10;
                let filled = (progress * bar_len as f64) as usize;
                let empty = bar_len - filled;
                let bar: String = "█".repeat(filled) + &"░".repeat(empty);

                dashboard.push_str(&format!("> - {} [`{}`] {}%\n", task.name, bar, percent));
            }
        }

        // Write to MD between anchors
        if let Ok(content) = fs::read_to_string(&self.md_path) {
            let start_marker = "<!-- MONITOR_START -->";
            let end_marker = "<!-- MONITOR_END -->";

            if let (Some(start), Some(end)) = (content.find(start_marker), content.find(end_marker))
            {
                let prefix = &content[..start + start_marker.len()];
                let suffix = &content[end..];

                let new_content = format!("{}{}{}", prefix, dashboard, suffix);

                // Only write if changed significantly or forced every 5s
                if content != new_content {
                    if let Ok(mut file) = fs::File::create(&self.md_path) {
                        let _ = file.write_all(new_content.as_bytes());
                    }
                }
            }
        }
    }

    fn log(&self, message: &str) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % 86400;

        let hrs = timestamp / 3600;
        let mins = (timestamp % 3600) / 60;
        let hrs_local = (hrs + 21) % 24;

        let log_line = format!("\n> `[{:02}:{:02}]` {}\n", hrs_local, mins, message);

        if let Ok(mut file) = OpenOptions::new().append(true).open(&self.md_path) {
            let _ = file.write_all(log_line.as_bytes());
        }
    }

    fn mark_done(&self, original_line: &str) {
        if let Ok(content) = fs::read_to_string(&self.md_path) {
            let new_line = original_line.replace("[x]", "[-]");
            let new_content = content.replace(original_line, &new_line);

            if let Ok(mut file) = fs::File::create(&self.md_path) {
                let _ = file.write_all(new_content.as_bytes());
            }
        }
    }
}

fn resolve_sentinel_bin() -> PathBuf {
    if let Ok(bin) = std::env::var("SENTINEL_BIN") {
        let p = PathBuf::from(bin);
        if p.is_file() {
            return p;
        }
    }

    let default_bin = PathBuf::from("/home/jnovoas/Obsidian/_Agentes/sentinel");
    if default_bin.is_file() {
        return default_bin;
    }

    PathBuf::from("sentinel")
}

impl AgentSPA for ControlAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn percibir(&mut self, _cortex: &CortexEngine) -> bool {
        let now = SystemTime::now();

        // 1. Read Commands Fast (every ~500ms)
        if let Ok(elapsed) = now.duration_since(self.last_read) {
            if elapsed.as_millis() > 500 {
                self.last_read = now;
                if let Ok(content) = fs::read_to_string(&self.md_path) {
                    let new_state = self.parse_markdown(&content);
                    let old_json = serde_json::to_string(&self.state).unwrap();
                    let new_json = serde_json::to_string(&new_state).unwrap();

                    if old_json != new_json {
                        self.state = new_state;
                        let _ = fs::write(&self.json_path, &new_json);
                    }
                }
            }
        }

        // 2. Update Dashboard Slower (every ~2000ms) to avoid flicker
        if let Ok(elapsed) = now.duration_since(self.last_dashboard_update) {
            if elapsed.as_millis() > 2000 {
                self.last_dashboard_update = now;
                self.update_dashboard();
            }
        }

        true
    }

    fn decidir(&mut self) -> String {
        "NO_OP".to_string()
    }

    fn actuar(&mut self, _action: String) {}
}
*/
