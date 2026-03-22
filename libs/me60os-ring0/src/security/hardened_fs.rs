use super::audit::SecurityAudit;
use super::capabilities::Capabilities;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct HardenedFs {
    capabilities: Arc<Capabilities>,
    audit: Arc<SecurityAudit>,
}

impl HardenedFs {
    pub fn new(capabilities: Arc<Capabilities>, audit: Arc<SecurityAudit>) -> Self {
        Self {
            capabilities,
            audit,
        }
    }

    pub fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path_ref = path.as_ref();

        if !self.capabilities.can_read(path_ref) {
            let _ = self.audit.log(
                "READ_DENIED",
                path_ref.to_string_lossy().as_ref(),
                "BLOCKED",
            );
            return Err(anyhow!(
                "SecurityViolation: Read access denied to {:?}",
                path_ref
            ));
        }

        match fs::read_to_string(path_ref) {
            Ok(content) => {
                let _ = self
                    .audit
                    .log("READ", path_ref.to_string_lossy().as_ref(), "SUCCESS");
                Ok(content)
            }
            Err(e) => {
                let _ = self.audit.log(
                    "READ_ERROR",
                    path_ref.to_string_lossy().as_ref(),
                    &e.to_string(),
                );
                Err(e.into())
            }
        }
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()> {
        let path_ref = path.as_ref();

        if !self.capabilities.can_write(path_ref) {
            let _ = self.audit.log(
                "WRITE_DENIED",
                path_ref.to_string_lossy().as_ref(),
                "BLOCKED",
            );
            return Err(anyhow!(
                "SecurityViolation: Write access denied to {:?}",
                path_ref
            ));
        }

        match fs::write(path_ref, contents) {
            Ok(_) => {
                let _ = self
                    .audit
                    .log("WRITE", path_ref.to_string_lossy().as_ref(), "SUCCESS");
                Ok(())
            }
            Err(e) => {
                let _ = self.audit.log(
                    "WRITE_ERROR",
                    path_ref.to_string_lossy().as_ref(),
                    &e.to_string(),
                );
                Err(e.into())
            }
        }
    }

    pub fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_ref = path.as_ref();
        // Check write permission for directory creation
        if !self.capabilities.can_write(path_ref) {
            let _ = self.audit.log(
                "MKDIR_DENIED",
                path_ref.to_string_lossy().as_ref(),
                "BLOCKED",
            );
            return Err(anyhow!(
                "SecurityViolation: Mkdir access denied to {:?}",
                path_ref
            ));
        }

        fs::create_dir_all(path_ref)?;
        let _ = self
            .audit
            .log("MKDIR", path_ref.to_string_lossy().as_ref(), "SUCCESS");
        Ok(())
    }

    pub fn append<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()> {
        let path_ref = path.as_ref();

        if !self.capabilities.can_write(path_ref) {
            let _ = self.audit.log(
                "APPEND_DENIED",
                path_ref.to_string_lossy().as_ref(),
                "BLOCKED",
            );
            return Err(anyhow!(
                "SecurityViolation: Append access denied to {:?}",
                path_ref
            ));
        }

        use std::io::Write;
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path_ref)
        {
            Ok(mut file) => match file.write_all(contents.as_ref()) {
                Ok(_) => {
                    let _ =
                        self.audit
                            .log("APPEND", path_ref.to_string_lossy().as_ref(), "SUCCESS");
                    Ok(())
                }
                Err(e) => {
                    let _ = self.audit.log(
                        "APPEND_ERROR",
                        path_ref.to_string_lossy().as_ref(),
                        &e.to_string(),
                    );
                    Err(e.into())
                }
            },
            Err(e) => {
                let _ = self.audit.log(
                    "APPEND_OPEN_ERROR",
                    path_ref.to_string_lossy().as_ref(),
                    &e.to_string(),
                );
                Err(e.into())
            }
        }
    }
}
