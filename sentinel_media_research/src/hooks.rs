use std::fs;
use std::path::Path;

pub trait ResearchHook: Send + Sync {
    fn on_search_start(&self, query: &str);
    fn on_search_complete(&self, results: &str);
    fn on_synthesis_start(&self, context: &str);
    fn on_synthesis_complete(&self, output: &Path);
    fn on_error(&self, error: &anyhow::Error);
}

pub struct NotificationHook {
    pub _ntfy_url: Option<String>,
}

impl ResearchHook for NotificationHook {
    fn on_search_start(&self, _query: &str) {
        self.log_progress(10, "Buscando en la Web...");
    }

    fn on_search_complete(&self, _results: &str) {
        self.log_progress(40, "Búsqueda web completada.");
    }

    fn on_synthesis_start(&self, _context: &str) {
        self.log_progress(60, "Sintetizando dossier...");
    }

    fn on_synthesis_complete(&self, _output: &Path) {
        self.log_progress(100, "Investigación completada.");
        // Notification logic would go here
    }

    fn on_error(&self, error: &anyhow::Error) {
        self.log_progress(0, &format!("ERROR: {}", error));
    }
}

impl NotificationHook {
    fn log_progress(&self, percent: u8, msg: &str) {
        // IPC for TUI
        let progress_file = "/dev/shm/sentinel_research_progress";
        let content = format!("{}|{}", percent, msg);
        let _ = fs::write(progress_file, content);
    }
}

pub struct LoggingHook;

impl ResearchHook for LoggingHook {
    fn on_search_start(&self, query: &str) {
        println!(
            "🔍 [HOOK] Search start: {}",
            query.chars().take(50).collect::<String>()
        );
    }
    fn on_search_complete(&self, _results: &str) {
        println!("✅ [HOOK] Search complete");
    }
    fn on_synthesis_start(&self, _context: &str) {
        println!("⚙️ [HOOK] Synthesis start");
    }
    fn on_synthesis_complete(&self, output: &Path) {
        println!("💎 [HOOK] Output saved to: {}", output.display());
    }
    fn on_error(&self, error: &anyhow::Error) {
        eprintln!("❌ [HOOK] Error: {}", error);
    }
}
