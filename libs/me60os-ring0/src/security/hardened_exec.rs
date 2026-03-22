use super::audit::SecurityAudit;
use super::capabilities::Capabilities;
use anyhow::{anyhow, Result};
use std::process::{Command, Output};
use std::sync::Arc;

#[derive(Clone)]
pub struct HardenedExec {
    capabilities: Arc<Capabilities>,
    audit: Arc<SecurityAudit>,
}

impl HardenedExec {
    pub fn new(capabilities: Arc<Capabilities>, audit: Arc<SecurityAudit>) -> Self {
        Self {
            capabilities,
            audit,
        }
    }

    pub fn exec(&self, program: &str, args: &[&str]) -> Result<Output> {
        // 1. Check Capabilities
        if !self.capabilities.can_execute(program) {
            let _ = self.audit.log("EXEC_DENIED", program, "BLOCKED");
            return Err(anyhow!(
                "SecurityViolation: Execution of '{}' not allowed.",
                program
            ));
        }

        // 2. Audit Intent
        let cmd_str = format!("{} {}", program, args.join(" "));
        let _ = self.audit.log("EXEC_ATTEMPT", &cmd_str, "PENDING");

        // 3. Optional Sandbox (Bubblewrap could be injected here)
        // For now, we execute directly but only if allowed.

        match Command::new(program).args(args).output() {
            Ok(output) => {
                let status = if output.status.success() {
                    "SUCCESS"
                } else {
                    "FAILED"
                };
                let _ = self.audit.log("EXEC_FINISH", &cmd_str, status);
                Ok(output)
            }
            Err(e) => {
                let _ = self.audit.log("EXEC_ERROR", &cmd_str, &e.to_string());
                Err(e.into())
            }
        }
    }
}
