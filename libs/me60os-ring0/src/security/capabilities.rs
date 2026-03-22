use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capabilities {
    pub allowed_read_paths: Vec<PathBuf>,
    pub allowed_write_paths: Vec<PathBuf>,
    pub allowed_commands: HashSet<String>,
    pub network_enabled: bool,
    pub allow_shell: bool, // DANGER: Only for debugging or specific high-trust agents
}

impl Capabilities {
    pub fn strict() -> Self {
        Self {
            allowed_read_paths: vec![],
            allowed_write_paths: vec![],
            allowed_commands: HashSet::new(),
            network_enabled: false,
            allow_shell: false,
        }
    }

    pub fn load_from_file(path: &Path) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or(Self::strict())
        } else {
            Self::strict()
        }
    }

    pub fn can_read(&self, path: &Path) -> bool {
        self.allowed_read_paths
            .iter()
            .any(|allowed| path.starts_with(allowed))
    }

    pub fn can_write(&self, path: &Path) -> bool {
        self.allowed_write_paths
            .iter()
            .any(|allowed| path.starts_with(allowed))
    }

    pub fn can_execute(&self, command: &str) -> bool {
        if self.allow_shell {
            return true;
        }
        self.allowed_commands.contains(command)
    }
}
