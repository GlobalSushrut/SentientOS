// SentientOS Linux Compatibility Layer
//
// This module provides compatibility with Linux applications by:
// - Translating Linux syscalls to SentientOS operations
// - Supporting ELF binary execution
// - Ensuring POSIX compliance
// - Managing Linux-compatible filesystem operations

pub mod syscall;
pub mod elf;
pub mod posix;
pub mod filesystem;
pub mod elf_loader;
pub mod compatibility;
pub mod cli;

use anyhow::Result;
use tracing::{info, warn};
use std::path::PathBuf;

use crate::core::constants;

/// Initialize the Linux compatibility layer
pub fn init() -> Result<()> {
    info!("Initializing Linux compatibility layer");
    
    // Create Linux compatibility directories
    let linux_dir = PathBuf::from(constants::ROOT_DIR).join(".linux");
    std::fs::create_dir_all(&linux_dir)?;
    
    let bin_dir = linux_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    
    let lib_dir = linux_dir.join("lib");
    std::fs::create_dir_all(&lib_dir)?;
    
    let etc_dir = linux_dir.join("etc");
    std::fs::create_dir_all(&etc_dir)?;
    
    let var_dir = linux_dir.join("var");
    std::fs::create_dir_all(&var_dir)?;
    
    // Initialize filesystem first as it's required by other subsystems
    filesystem::init()?;
    
    // Initialize POSIX compatibility layer
    posix::init()?;
    
    // Initialize syscall translation
    syscall::init()?;
    
    // Initialize the ELF binary execution subsystem
    elf::init()?;
    
    info!("Linux compatibility layer initialized successfully");
    Ok(())
}

/// Shutdown the Linux compatibility layer
pub fn shutdown() -> Result<()> {
    info!("Shutting down Linux compatibility layer");
    
    // Shutdown components in reverse order
    filesystem::shutdown()?;
    posix::shutdown()?;
    elf::shutdown()?;
    syscall::shutdown()?;
    
    info!("Linux compatibility layer shutdown complete");
    Ok(())
}

/// Execute a Linux ELF binary
pub fn execute_binary(binary_path: &str, args: Vec<String>) -> Result<i32> {
    info!("Executing Linux binary: {} with args: {:?}", binary_path, args);
    
    // Translate path if needed
    let translated_path = filesystem::translate_to_linux_path(binary_path);
    
    // Check if file exists and is accessible
    if !filesystem::path_exists(&translated_path) {
        return Err(anyhow::anyhow!("Binary not found: {}", binary_path));
    }
    
    // Load the ELF binary
    let elf_binary = elf::load_binary(&translated_path)?;
    
    // Create execution context
    let mut context = elf::create_execution_context(elf_binary, args)?;
    
    // Execute the binary
    let exit_code = elf::execute(&mut context)?;
    
    info!("Linux binary execution completed with exit code: {}", exit_code);
    Ok(exit_code)
}

/// Check if a file is a valid Linux binary
pub fn is_linux_binary(path: &str) -> Result<bool> {
    elf::is_valid_elf(path)
}

/// Register a syscall handler for a specific Linux syscall
pub fn register_syscall_handler(syscall_number: i32, handler: syscall::SyscallHandler) -> Result<()> {
    syscall::register_handler(syscall_number, handler)
}

/// Get the Linux compatibility mode
pub fn get_compatibility_mode() -> CompatibilityMode {
    // Check if we're running in strict compatibility mode
    if std::env::var("SENTIENT_LINUX_STRICT").is_ok() {
        CompatibilityMode::Strict
    } else {
        CompatibilityMode::Enhanced
    }
}

/// Linux compatibility mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityMode {
    /// Strict Linux compatibility (no enhancements)
    Strict,
    
    /// Enhanced compatibility with SentientOS features
    Enhanced,
}
