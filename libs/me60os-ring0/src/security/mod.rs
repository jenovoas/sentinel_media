pub mod audit;
pub mod capabilities;
pub mod hardened_exec;
pub mod hardened_fs;

// Re-exports
pub use audit::SecurityAudit;
pub use capabilities::Capabilities;
pub use hardened_exec::HardenedExec;
pub use hardened_fs::HardenedFs;

use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct SecurityKernel {
    pub capabilities: Arc<Capabilities>,
    pub audit: Arc<SecurityAudit>,
    pub fs: HardenedFs,
    pub exec: HardenedExec,
}

impl SecurityKernel {
    pub fn new(manifest_path: &Path, audit_log_path: &Path) -> Self {
        let caps = Arc::new(Capabilities::load_from_file(manifest_path));
        let audit = Arc::new(SecurityAudit::new(audit_log_path));

        Self {
            capabilities: caps.clone(),
            audit: audit.clone(),
            fs: HardenedFs::new(caps.clone(), audit.clone()),
            exec: HardenedExec::new(caps.clone(), audit.clone()),
        }
    }
}
