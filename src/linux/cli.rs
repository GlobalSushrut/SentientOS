// SentientOS Linux Compatibility CLI Module
// Provides the bridge between CLI and Linux compatibility layer

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, debug, warn, error};

pub use crate::cli::linux::LinuxCommands;
use crate::linux::{compatibility, elf_loader};

/// Handle Linux compatibility CLI commands
pub fn handle_command(cmd: &LinuxCommands) -> Result<()> {
    // This function delegates to the implementation in cli::linux
    crate::cli::linux::handle_command(cmd)
}

/// Convert Linux process information to human-readable format
pub fn format_process_info(process: &compatibility::LinuxProcessInfo) -> String {
    let status = match &process.status {
        compatibility::ProcessStatus::Running => "running".to_string(),
        compatibility::ProcessStatus::Stopped => "stopped".to_string(),
        compatibility::ProcessStatus::Exited(code) => format!("exited({})", code),
        compatibility::ProcessStatus::Failed(code) => format!("failed({})", code),
    };

    let container_info = if let Some(container) = &process.container_name {
        format!(" in container {}", container)
    } else {
        String::new()
    };

    format!(
        "Process {} [{}]: {} {}{}",
        process.id,
        status,
        process.path,
        process.args.join(" "),
        container_info
    )
}

/// Kill a process by ID
pub fn kill_process(pid: u32, force: bool) -> Result<()> {
    // Convert numeric PID to process ID string format
    // This is a simplification; in a real implementation we would map between
    // the numeric PIDs shown to users and the internal process IDs
    let process_id = format!("proc-{}", pid);
    
    if force {
        info!("Force killing Linux process: {}", process_id);
    } else {
        info!("Terminating Linux process: {}", process_id);
    }
    
    compatibility::stop_process(&process_id)
}
