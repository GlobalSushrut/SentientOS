// SentientOS Filesystem Structure
// Handles initialization and maintenance of the file system structure

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

use crate::core::constants;

/// Initialize the filesystem structure
pub fn init() -> Result<()> {
    info!("Initializing SentientOS filesystem structure");
    
    // Create the root directory if it doesn't exist
    let root_dir = PathBuf::from(constants::ROOT_DIR);
    fs::create_dir_all(&root_dir)?;
    
    // Create standard system directories
    create_system_directories()?;
    
    // Create standard config files
    create_default_configs()?;
    
    // Set up directory permissions
    setup_permissions()?;
    
    info!("SentientOS filesystem structure initialized successfully");
    Ok(())
}

/// Create the standard system directories
fn create_system_directories() -> Result<()> {
    debug!("Creating standard system directories");
    
    let root_dir = PathBuf::from(constants::ROOT_DIR);
    
    // Define the system directory structure
    let directories = [
        // Core system directories
        ".heal",             // Healing subsystem
        ".heal/snapshots",   // System snapshots
        ".heal/recovery",    // Recovery files
        ".panic",            // Panic subsystem
        ".panic/logs",       // Panic logs
        ".panic/fallback",   // Fallback states
        ".zk",               // Zero-knowledge proof subsystem
        ".zk/contracts",     // ZK contracts
        ".zk/proofs",        // Generated proofs
        ".matrixbox",        // Container runtime
        ".matrixbox/images", // Container images
        ".matrixbox/data",   // Container persistent data
        ".boot",             // Boot subsystem
        ".boot/zig",         // Zig bootloader files
        ".boot/config",      // Boot configuration
        ".auth",             // Authentication subsystem
        ".auth/keys",        // Cryptographic keys
        ".auth/policies",    // Access policies
        ".lock",             // Locking subsystem
        ".lock/resources",   // Resource locks
        ".gossip",           // Gossip synchronization
        ".gossip/peers",     // Peer information
        ".gossip/sync",      // Sync state
        ".intent",           // Developer intent system
        ".intent/sessions",  // Recorded sessions
        ".intent/replay",    // Replay data
        ".cli",              // CLI configuration
        ".runtime",          // Runtime state
        ".container",        // Container storage
        ".config",           // System configuration
        "bin",               // Executables
        "lib",               // Libraries
        "data",              // User data
        "tmp",               // Temporary files
        "logs",              // System logs
    ];
    
    // Create each directory
    for dir in &directories {
        let path = root_dir.join(dir);
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create directory: {:?}", path))?;
        debug!("Created directory: {:?}", path);
    }
    
    Ok(())
}

/// Create default configuration files
fn create_default_configs() -> Result<()> {
    debug!("Creating default configuration files");
    
    let root_dir = PathBuf::from(constants::ROOT_DIR);
    
    // System configuration
    let system_config = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "initialized_at": chrono::Utc::now().to_rfc3339(),
        "node_id": generate_node_id(),
        "subsystems": {
            "heal": { "enabled": true, "snapshot_interval_minutes": 60 },
            "panic": { "enabled": true, "max_recovery_attempts": 3 },
            "matrixbox": { "enabled": true, "max_containers": 50 },
            "zk": { "enabled": true },
            "gossip": { "enabled": true },
            "intent": { "enabled": true },
        }
    });
    
    let system_config_path = root_dir.join(".config").join("system.json");
    fs::write(&system_config_path, serde_json::to_string_pretty(&system_config)?)?;
    debug!("Created system config: {:?}", system_config_path);
    
    // Security policy
    let security_policy = serde_json::json!({
        "default_container_policy": "restricted",
        "zk_verification_required": true,
        "peer_authentication_required": true,
        "audit_logging_enabled": true
    });
    
    let security_policy_path = root_dir.join(".config").join("security.json");
    fs::write(&security_policy_path, serde_json::to_string_pretty(&security_policy)?)?;
    debug!("Created security policy: {:?}", security_policy_path);
    
    Ok(())
}

/// Set up directory permissions
fn setup_permissions() -> Result<()> {
    debug!("Setting up directory permissions");
    
    // In a real implementation, we would use proper file system permissions
    // For now, we'll just create a permissions manifest file
    
    let root_dir = PathBuf::from(constants::ROOT_DIR);
    
    // Define permission structure
    let permissions = serde_json::json!({
        ".zk": {
            "user_read": true,
            "user_write": true,
            "system_read": true,
            "system_write": true,
            "container_read": false,
            "container_write": false
        },
        ".matrixbox": {
            "user_read": true,
            "user_write": false,
            "system_read": true,
            "system_write": true,
            "container_read": true,
            "container_write": false
        },
        "data": {
            "user_read": true,
            "user_write": true,
            "system_read": true,
            "system_write": true,
            "container_read": true,
            "container_write": true
        }
        // More permissions would be defined here
    });
    
    let permissions_path = root_dir.join(".config").join("permissions.json");
    fs::write(&permissions_path, serde_json::to_string_pretty(&permissions)?)?;
    debug!("Created permissions manifest: {:?}", permissions_path);
    
    Ok(())
}

/// Generate a unique node ID
fn generate_node_id() -> String {
    use rand::{thread_rng, Rng};
    use blake3;
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();
    
    let mut rng = thread_rng();
    let random_bytes: [u8; 8] = rng.gen();
    
    // Hash timestamp and random bytes for uniqueness
    let mut hasher = blake3::Hasher::new();
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(&random_bytes);
    
    let hash = hasher.finalize();
    let node_id = hash.to_hex().to_string();
    
    // Use first 16 chars of the hash
    node_id[..16].to_string()
}

/// Check if the filesystem structure is properly initialized
pub fn check_structure() -> Result<bool> {
    debug!("Checking filesystem structure");
    
    let root_dir = PathBuf::from(constants::ROOT_DIR);
    
    // Check essential directories
    let essential_dirs = [
        ".config",
        ".zk",
        ".matrixbox",
        ".heal",
        ".gossip",
    ];
    
    for dir in &essential_dirs {
        let path = root_dir.join(dir);
        if !path.exists() || !path.is_dir() {
            warn!("Essential directory missing: {:?}", path);
            return Ok(false);
        }
    }
    
    // Check essential config files
    let system_config_path = root_dir.join(".config").join("system.json");
    if !system_config_path.exists() {
        warn!("Essential config file missing: {:?}", system_config_path);
        return Ok(false);
    }
    
    debug!("Filesystem structure check passed");
    Ok(true)
}

/// Repair filesystem structure if needed
pub fn repair_structure() -> Result<()> {
    debug!("Repairing filesystem structure");
    
    if check_structure()? {
        debug!("Filesystem structure is already valid, no repair needed");
        return Ok(());
    }
    
    // Recreate system directories
    create_system_directories()?;
    
    // Recreate config files if missing
    let root_dir = PathBuf::from(constants::ROOT_DIR);
    let system_config_path = root_dir.join(".config").join("system.json");
    if !system_config_path.exists() {
        create_default_configs()?;
    }
    
    // Reapply permissions
    setup_permissions()?;
    
    info!("Filesystem structure repaired");
    Ok(())
}
