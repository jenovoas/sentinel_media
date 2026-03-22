use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct TelosContext {
    pub mission: String,
    pub goals: Vec<String>,
    pub active_projects: Vec<String>,
    pub beliefs: Vec<String>,
}

impl TelosContext {
    pub fn new() -> Self {
        Self {
            mission: "No definida en el Vault.".to_string(),
            goals: Vec::new(),
            active_projects: Vec::new(),
            beliefs: Vec::new(),
        }
    }

    pub fn load_from_vault(vault_path: &Path) -> Result<Self> {
        let mut context = Self::new();

        // Search for key files
        for entry in WalkDir::new(vault_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let filename = path.file_name().unwrap().to_string_lossy().to_uppercase();

                if filename.contains("MISSION") || filename.contains("MISIÓN") {
                    context.mission = fs::read_to_string(path)?;
                } else if filename.contains("GOALS") || filename.contains("OBJETIVOS") {
                    context.goals.push(fs::read_to_string(path)?);
                } else if filename.contains("PROJECTS") || filename.contains("PROYECTOS") {
                    context.active_projects.push(fs::read_to_string(path)?);
                } else if filename.contains("BELIEFS") || filename.contains("CREENCIAS") {
                    context.beliefs.push(fs::read_to_string(path)?);
                } else if filename.contains("CONTROL_ROOM") {
                    // Especial para Sentinel
                    if context.active_projects.is_empty() {
                        context.active_projects.push(fs::read_to_string(path)?);
                    }
                }
            }
        }

        // Final fallback: check for Machete files if mission is empty
        if context.mission.contains("No definida") {
            for entry in WalkDir::new(vault_path)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let filename = entry.file_name().to_string_lossy();
                if filename.contains("Machete") {
                    context.mission = format!(
                        "Derivada de Machete:\n{}",
                        fs::read_to_string(entry.path())?
                    );
                    break;
                }
            }
        }

        Ok(context)
    }

    pub fn inject_into_prompt(&self, user_query: &str) -> String {
        let mut context_str = String::from("--- IDENTITY CONTEXT (TELOS) ---\n");
        context_str.push_str(&format!("MISSION:\n{}\n\n", self.mission));

        if !self.goals.is_empty() {
            context_str.push_str("GOALS:\n");
            for goal in &self.goals {
                context_str.push_str(&format!(
                    "- {}\n",
                    goal.chars().take(200).collect::<String>()
                ));
            }
            context_str.push('\n');
        }

        if !self.active_projects.is_empty() {
            context_str.push_str("ACTIVE PROJECTS:\n");
            for project in &self.active_projects {
                context_str.push_str(&format!(
                    "- {}\n",
                    project.chars().take(200).collect::<String>()
                ));
            }
            context_str.push('\n');
        }

        context_str.push_str("--- END IDENTITY CONTEXT ---\n\n");
        context_str.push_str(&format!("USER QUERY: {}", user_query));

        context_str
    }
}
