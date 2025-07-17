use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use super::SnapshotInfo;
use crate::core::constants;

/// Snapshot metadata
#[derive(Debug, Serialize, Deserialize)]
struct SnapshotMetadata {
    /// Snapshot ID
    id: String,
    
    /// Timestamp when the snapshot was taken
    timestamp: u64,
    
    /// Reason for taking the snapshot
    reason: String,
    
    /// Components included in the snapshot
    components: Vec<String>,
    
    /// Hash of the snapshot contents
    content_hash: String,
}

/// Initialize the snapshot system
pub fn init() -> Result<()> {
    info!("Initializing snapshot system");
    
    let snapshot_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("snapshots");
    
    fs::create_dir_all(&snapshot_dir)
        .context("Failed to create snapshot directory")?;
    
    info!("Snapshot system initialized");
    Ok(())
}

/// Shutdown the snapshot system
pub fn shutdown() -> Result<()> {
    info!("Shutting down snapshot system");
    
    // Nothing specific to shut down
    
    info!("Snapshot system shutdown complete");
    Ok(())
}

/// Create a new system snapshot
pub fn create_snapshot(id: &str, reason: &str) -> Result<()> {
    info!("Creating snapshot: {} - {}", id, reason);
    
    // Create snapshot directory
    let snapshot_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("snapshots")
        .join(id);
    
    fs::create_dir_all(&snapshot_dir)
        .with_context(|| format!("Failed to create snapshot directory: {}", id))?;
    
    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get system time")?
        .as_secs();
    
    // Components to snapshot
    let components = vec![
        "core",
        "zk",
        "containers",
        "runtime",
        "auth",
        "linux",
    ];
    
    // Take snapshots of each component
    for component in &components {
        snapshot_component(component, &snapshot_dir)
            .with_context(|| format!("Failed to snapshot component: {}", component))?;
    }
    
    // Calculate content hash
    let content_hash = calculate_snapshot_hash(&snapshot_dir)?;
    
    // Create metadata
    let metadata = SnapshotMetadata {
        id: id.to_string(),
        timestamp,
        reason: reason.to_string(),
        components: components.iter().map(|s| s.to_string()).collect(),
        content_hash: content_hash.clone(),
    };
    
    // Save metadata
    let metadata_path = snapshot_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .context("Failed to serialize snapshot metadata")?;
    
    fs::write(&metadata_path, metadata_json)
        .context("Failed to write snapshot metadata")?;
    
    info!("Snapshot created successfully: {}", id);
    Ok(())
}

/// Take a snapshot of a specific component
fn snapshot_component(component: &str, snapshot_dir: &Path) -> Result<()> {
    debug!("Snapshotting component: {}", component);
    
    let component_dir = snapshot_dir.join(component);
    fs::create_dir_all(&component_dir)
        .with_context(|| format!("Failed to create component directory: {}", component))?;
    
    // Determine the source path based on the component
    let source_path = match component {
        "core" => PathBuf::from(constants::ROOT_DIR).join(constants::CORE_DIR),
        "zk" => PathBuf::from(constants::ROOT_DIR).join(constants::ZK_DIR),
        "containers" => PathBuf::from(constants::ROOT_DIR).join(constants::CONTAINER_DIR),
        "runtime" => PathBuf::from(constants::ROOT_DIR).join(constants::RUNTIME_DIR),
        "auth" => PathBuf::from(constants::ROOT_DIR).join(constants::AUTH_DIR),
        "linux" => PathBuf::from(constants::ROOT_DIR).join(".linux"),
        _ => anyhow::bail!("Unknown component: {}", component),
    };
    
    if !source_path.exists() {
        warn!("Component path does not exist: {:?}", source_path);
        return Ok(());
    }
    
    // For each component, we'll save:
    // 1. Configuration files
    // 2. State files
    // 3. Component-specific data
    
    match component {
        "core" => {
            // Core configuration
            let config_path = source_path.join("config.yaml");
            if config_path.exists() {
                copy_file(&config_path, &component_dir.join("config.yaml"))?;
            }
            
            // Core state
            let state_path = source_path.join("state.json");
            if state_path.exists() {
                copy_file(&state_path, &component_dir.join("state.json"))?;
            }
        },
        "zk" => {
            // ZK contracts
            let contracts_path = source_path.join("contracts");
            if contracts_path.exists() {
                copy_directory(&contracts_path, &component_dir.join("contracts"))?;
            }
            
            // ZK verification keys
            let keys_path = source_path.join("keys");
            if keys_path.exists() {
                copy_directory(&keys_path, &component_dir.join("keys"))?;
            }
        },
        "containers" => {
            // Container registry
            let registry_path = source_path.join("registry");
            if registry_path.exists() {
                copy_directory(&registry_path, &component_dir.join("registry"))?;
            }
            
            // Active container state (but not the actual containers)
            let registry_file = registry_path.join("registry.json");
            if registry_file.exists() {
                copy_file(&registry_file, &component_dir.join("registry.json"))?;
            }
        },
        "runtime" => {
            // Runtime state
            let state_path = source_path.join("state.json");
            if state_path.exists() {
                copy_file(&state_path, &component_dir.join("state.json"))?;
            }
            
            // Runtime logs (last 10 only)
            let logs_path = source_path.join("logs");
            if logs_path.exists() {
                let log_files = fs::read_dir(&logs_path)?
                    .filter_map(Result::ok)
                    .filter(|entry| {
                        entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) &&
                        entry.file_name().to_string_lossy().ends_with(".log")
                    })
                    .collect::<Vec<_>>();
                
                // Sort by modified time, most recent first
                let mut sorted_logs = log_files;
                sorted_logs.sort_by(|a, b| {
                    let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| UNIX_EPOCH);
                    let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| UNIX_EPOCH);
                    b_time.cmp(&a_time)
                });
                
                // Copy only the 10 most recent logs
                let logs_dest = component_dir.join("logs");
                fs::create_dir_all(&logs_dest)?;
                
                for (i, log) in sorted_logs.iter().take(10).enumerate() {
                    let dest = logs_dest.join(format!("log_{}.log", i));
                    copy_file(&log.path(), &dest)?;
                }
            }
        },
        "auth" => {
            // Auth configuration
            let config_path = source_path.join("config.yaml");
            if config_path.exists() {
                copy_file(&config_path, &component_dir.join("config.yaml"))?;
            }
            
            // Auth keys (excluding private keys)
            let keys_path = source_path.join("keys");
            if keys_path.exists() {
                let keys_dest = component_dir.join("keys");
                fs::create_dir_all(&keys_dest)?;
                
                // Only copy public keys
                if let Ok(entries) = fs::read_dir(&keys_path) {
                    for entry in entries.filter_map(Result::ok) {
                        let file_name = entry.file_name();
                        let name_str = file_name.to_string_lossy();
                        
                        // Only copy public keys or non-sensitive data
                        if name_str.contains("public") || name_str.ends_with(".pub") {
                            copy_file(&entry.path(), &keys_dest.join(file_name))?;
                        }
                    }
                }
            }
        },
        "linux" => {
            // Linux compatibility layer configuration
            let etc_path = source_path.join("etc");
            if etc_path.exists() {
                copy_directory(&etc_path, &component_dir.join("etc"))?;
            }
        },
        _ => {}
    }
    
    debug!("Component snapshot complete: {}", component);
    Ok(())
}

/// Calculate a hash of the snapshot contents
fn calculate_snapshot_hash(snapshot_dir: &Path) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    
    // Hash all files in the snapshot directory recursively
    hash_directory_recursive(snapshot_dir, &mut hasher)?;
    
    // Finalize hash
    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

/// Hash a directory recursively
fn hash_directory_recursive(dir: &Path, hasher: &mut blake3::Hasher) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Hash the path itself
        hasher.update(path.to_string_lossy().as_bytes());
        
        if path.is_dir() {
            // Recursively hash subdirectories
            hash_directory_recursive(&path, hasher)?;
        } else if path.is_file() {
            // Hash file contents
            let mut file = File::open(&path)?;
            let mut buffer = [0; 8192];
            
            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                
                hasher.update(&buffer[..bytes_read]);
            }
        }
    }
    
    Ok(())
}

/// Copy a file
fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    debug!("Copying file: {:?} -> {:?}", src, dst);
    
    // Ensure the parent directory exists
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::copy(src, dst)?;
    Ok(())
}

/// Copy a directory recursively
fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    debug!("Copying directory: {:?} -> {:?}", src, dst);
    
    // Create destination directory
    fs::create_dir_all(dst)?;
    
    // Copy all entries
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        
        if path.is_dir() {
            // Recursively copy subdirectories
            copy_directory(&path, &dest_path)?;
        } else {
            // Copy files
            copy_file(&path, &dest_path)?;
        }
    }
    
    Ok(())
}

/// List all available snapshots
pub fn list_snapshots() -> Result<Vec<SnapshotInfo>> {
    let snapshot_base = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("snapshots");
    
    if !snapshot_base.exists() {
        return Ok(Vec::new());
    }
    
    let mut snapshots = Vec::new();
    
    for entry in fs::read_dir(&snapshot_base)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let metadata_path = path.join("metadata.json");
            
            if metadata_path.exists() {
                let metadata_json = fs::read_to_string(&metadata_path)?;
                let metadata: SnapshotMetadata = serde_json::from_str(&metadata_json)?;
                
                snapshots.push(SnapshotInfo {
                    id: metadata.id,
                    timestamp: metadata.timestamp,
                    reason: metadata.reason,
                    path,
                    hash: metadata.content_hash,
                });
            }
        }
    }
    
    // Sort snapshots by timestamp, newest first
    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Ok(snapshots)
}

/// Get a specific snapshot by ID
pub fn get_snapshot(id: &str) -> Result<Option<SnapshotInfo>> {
    let snapshots = list_snapshots()?;
    
    for snapshot in snapshots {
        if snapshot.id == id {
            return Ok(Some(snapshot));
        }
    }
    
    Ok(None)
}

/// Delete a snapshot
pub fn delete_snapshot(id: &str) -> Result<()> {
    info!("Deleting snapshot: {}", id);
    
    let snapshot_path = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("snapshots")
        .join(id);
    
    if !snapshot_path.exists() {
        anyhow::bail!("Snapshot not found: {}", id);
    }
    
    // Remove the snapshot directory
    fs::remove_dir_all(&snapshot_path)
        .with_context(|| format!("Failed to delete snapshot: {}", id))?;
    
    info!("Snapshot deleted: {}", id);
    Ok(())
}
