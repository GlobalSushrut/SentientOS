// SentientOS Core Module
// Handles core system functionality

pub mod fs;
pub mod error;

/// Core system constants
pub mod constants {
    use std::path::Path;

    /// Root directory of SentientOS
    pub const ROOT_DIR: &str = "/home/umesh/Sentinent_os";
    
    /// Core system directories
    pub const RUNTIME_DIR: &str = ".runtime";
    pub const LOCK_DIR: &str = ".lock";
    pub const AUTH_DIR: &str = ".auth";
    pub const BROWSER_DIR: &str = ".browser";
    pub const CONTAINER_DIR: &str = ".container";
    pub const HEAL_DIR: &str = ".heal";
    pub const GOSSIP_DIR: &str = ".gossip";
    pub const INTENT_DIR: &str = ".intent";
    pub const PANIC_DIR: &str = ".panic";
    pub const ZERO_DIR: &str = ".zero";
    pub const UNSECURE_DIR: &str = ".unsecure";
    
    /// Get the absolute path to a SentientOS directory
    pub fn get_path(dir: &str) -> String {
        Path::new(ROOT_DIR).join(dir).to_string_lossy().to_string()
    }
}
