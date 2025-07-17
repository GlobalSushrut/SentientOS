use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs;

use crate::core::constants;

/// Initialize the recovery system
pub fn init() -> Result<()> {
    info!("Initializing recovery system");
    
    let recovery_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("recovery");
    
    fs::create_dir_all(&recovery_dir)
        .context("Failed to create recovery directory")?;
    
    info!("Recovery system initialized");
    Ok(())
}

/// Shutdown the recovery system
pub fn shutdown() -> Result<()> {
    info!("Shutting down recovery system");
    
    // Nothing specific to shut down
    
    info!("Recovery system shutdown complete");
    Ok(())
}

/// Recover from a snapshot
pub fn recover_from_snapshot(snapshot_id: &str) -> Result<()> {
    info!("Recovering from snapshot: {}", snapshot_id);
    
    // Verify snapshot exists
    let snapshot_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("snapshots")
        .join(snapshot_id);
    
    if !snapshot_dir.exists() {
        return Err(anyhow::anyhow!("Snapshot not found: {}", snapshot_id));
    }
    
    // Create a recovery log
    let recovery_log = create_recovery_log(snapshot_id)?;
    
    // Perform component recovery in order
    recover_component("core", &snapshot_dir, &recovery_log)?;
    recover_component("zk", &snapshot_dir, &recovery_log)?;
    recover_component("auth", &snapshot_dir, &recovery_log)?;
    recover_component("containers", &snapshot_dir, &recovery_log)?;
    recover_component("runtime", &snapshot_dir, &recovery_log)?;
    recover_component("linux", &snapshot_dir, &recovery_log)?;
    
    info!("Recovery complete from snapshot: {}", snapshot_id);
    Ok(())
}

/// Create a recovery log file
fn create_recovery_log(snapshot_id: &str) -> Result<PathBuf> {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let log_name = format!("recovery-{}-{}.log", snapshot_id, timestamp);
    
    let log_path = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("logs")
        .join(log_name);
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Create the log file with initial header
    let header = format!("# SentientOS Recovery Log\n# Snapshot: {}\n# Time: {}\n\n", 
                       snapshot_id, timestamp);
    fs::write(&log_path, header)?;
    
    Ok(log_path)
}

/// Recover a specific component
fn recover_component(component: &str, snapshot_dir: &Path, recovery_log: &Path) -> Result<()> {
    debug!("Recovering component: {}", component);
    
    // Source path in snapshot
    let component_source = snapshot_dir.join(component);
    
    if !component_source.exists() {
        warn!("Component not found in snapshot: {}", component);
        log_recovery_event(recovery_log, component, "SKIPPED", "Component not in snapshot")?;
        return Ok(());
    }
    
    // Target path in system
    let target_path = match component {
        "core" => PathBuf::from(constants::ROOT_DIR).join(constants::CORE_DIR),
        "zk" => PathBuf::from(constants::ROOT_DIR).join(constants::ZK_DIR),
        "containers" => PathBuf::from(constants::ROOT_DIR).join(constants::CONTAINER_DIR),
        "runtime" => PathBuf::from(constants::ROOT_DIR).join(constants::RUNTIME_DIR),
        "auth" => PathBuf::from(constants::ROOT_DIR).join(constants::AUTH_DIR),
        "linux" => PathBuf::from(constants::ROOT_DIR).join(".linux"),
        _ => {
            warn!("Unknown component: {}", component);
            log_recovery_event(recovery_log, component, "ERROR", "Unknown component")?;
            return Err(anyhow::anyhow!("Unknown component: {}", component));
        },
    };
    
    // Ensure target directory exists
    fs::create_dir_all(&target_path)?;
    
    // Restore files from snapshot
    restore_files(&component_source, &target_path, recovery_log, component)?;
    
    log_recovery_event(recovery_log, component, "SUCCESS", "Component recovered")?;
    debug!("Component recovery complete: {}", component);
    Ok(())
}

/// Log a recovery event
fn log_recovery_event(log_path: &Path, component: &str, status: &str, message: &str) -> Result<()> {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] {} - {}: {}\n", timestamp, component, status, message);
    
    // Append to the log file
    fs::OpenOptions::new()
        .append(true)
        .open(log_path)?
        .write_all(log_entry.as_bytes())?;
    
    Ok(())
}

/// Restore files from a snapshot to the target system
fn restore_files(source: &Path, target: &Path, log_path: &Path, component: &str) -> Result<()> {
    debug!("Restoring files from {:?} to {:?}", source, target);
    
    // Check if source exists
    if !source.exists() {
        log_recovery_event(log_path, component, "ERROR", &format!("Source path does not exist: {:?}", source))?;
        return Err(anyhow::anyhow!("Source path does not exist: {:?}", source));
    }
    
    // Create backup of target if it exists and has content
    if target.exists() {
        backup_target_before_restore(target, component)?;
    }
    
    // Track progress
    let mut success_count = 0;
    let mut error_count = 0;
    
    // Copy files based on component-specific rules
    match component {
        "core" => {
            // For core, we restore config and state files
            restore_specific_files(source, target, &["config.yaml", "state.json"], 
                                 log_path, component, &mut success_count, &mut error_count)?;
        },
        "zk" => {
            // For ZK, restore contracts and keys directories
            restore_directory(source.join("contracts"), target.join("contracts"), 
                           log_path, component, &mut success_count, &mut error_count)?;
            
            restore_directory(source.join("keys"), target.join("keys"), 
                           log_path, component, &mut success_count, &mut error_count)?;
        },
        "containers" => {
            // For containers, just restore the registry file, not actual containers
            restore_specific_files(source, target, &["registry.json"], 
                                 log_path, component, &mut success_count, &mut error_count)?;
        },
        "runtime" => {
            // For runtime, restore state but not logs
            restore_specific_files(source, target, &["state.json"], 
                                 log_path, component, &mut success_count, &mut error_count)?;
        },
        "auth" => {
            // For auth, restore config and public keys only
            restore_specific_files(source, target, &["config.yaml"], 
                                 log_path, component, &mut success_count, &mut error_count)?;
            
            // Public keys are in a directory
            let src_keys = source.join("keys");
            let tgt_keys = target.join("keys");
            
            if src_keys.exists() {
                // Only restore public keys
                restore_directory_with_filter(src_keys, tgt_keys, |name| {
                    name.contains("public") || name.ends_with(".pub")
                }, log_path, component, &mut success_count, &mut error_count)?;
            }
        },
        "linux" => {
            // For Linux compatibility, restore etc directory
            restore_directory(source.join("etc"), target.join("etc"), 
                           log_path, component, &mut success_count, &mut error_count)?;
        },
        _ => {
            // For unknown components, just try to restore everything
            restore_directory(source.clone(), target.clone(), 
                           log_path, component, &mut success_count, &mut error_count)?;
        }
    }
    
    // Log summary
    log_recovery_event(log_path, component, "SUMMARY", 
                    &format!("Restored {} files, {} errors", success_count, error_count))?;
    
    debug!("File restoration complete for component {}: {} successes, {} errors", 
         component, success_count, error_count);
    
    Ok(())
}

/// Create a backup of the target directory before restoration
fn backup_target_before_restore(target: &Path, component: &str) -> Result<()> {
    debug!("Creating backup of target before restore: {:?}", target);
    
    if !target.exists() {
        return Ok(());
    }
    
    // Create backup directory
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let backup_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("backups")
        .join(format!("{}-{}", component, timestamp));
    
    fs::create_dir_all(&backup_dir)?;
    
    // Copy contents to backup
    copy_directory_contents(target, &backup_dir)?;
    
    debug!("Backup created at {:?}", backup_dir);
    Ok(())
}

/// Copy directory contents recursively
fn copy_directory_contents(source: &Path, target: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    
    // Create target if it doesn't exist
    fs::create_dir_all(target)?;
    
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let target_path = target.join(&file_name);
        
        if path.is_dir() {
            // Recursively copy directory
            copy_directory_contents(&path, &target_path)?;
        } else {
            // Copy file
            fs::copy(&path, &target_path)?;
        }
    }
    
    Ok(())
}

/// Restore specific files from source to target
fn restore_specific_files(
    source: &Path, 
    target: &Path, 
    files: &[&str],
    log_path: &Path,
    component: &str,
    success_count: &mut usize,
    error_count: &mut usize
) -> Result<()> {
    for file in files {
        let src_file = source.join(file);
        let tgt_file = target.join(file);
        
        // Skip if source file doesn't exist
        if !src_file.exists() {
            continue;
        }
        
        // Ensure target parent directory exists
        if let Some(parent) = tgt_file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        match fs::copy(&src_file, &tgt_file) {
            Ok(_) => {
                *success_count += 1;
                log_recovery_event(log_path, component, "RESTORED", &format!("File {}", file))?;
            },
            Err(e) => {
                *error_count += 1;
                log_recovery_event(log_path, component, "ERROR", &format!("Failed to restore {}: {}", file, e))?;
            }
        }
    }
    
    Ok(())
}

/// Restore an entire directory recursively
fn restore_directory(
    source: &Path, 
    target: &Path,
    log_path: &Path,
    component: &str,
    success_count: &mut usize,
    error_count: &mut usize
) -> Result<()> {
    // Skip if source doesn't exist
    if !source.exists() {
        return Ok(());
    }
    
    // Create target if it doesn't exist
    fs::create_dir_all(target)?;
    
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let target_path = target.join(&file_name);
        
        if path.is_dir() {
            // Recursively restore directory
            restore_directory(&path, &target_path, log_path, component, success_count, error_count)?;
        } else {
            // Restore file
            match fs::copy(&path, &target_path) {
                Ok(_) => {
                    *success_count += 1;
                    log_recovery_event(log_path, component, "RESTORED", 
                                    &format!("File {:?}", path.file_name().unwrap_or_default()))?;
                },
                Err(e) => {
                    *error_count += 1;
                    log_recovery_event(log_path, component, "ERROR", 
                                    &format!("Failed to restore {:?}: {}", 
                                          path.file_name().unwrap_or_default(), e))?;
                }
            }
        }
    }
    
    Ok(())
}

/// Restore a directory with a filter function
fn restore_directory_with_filter<F>(
    source: &Path, 
    target: &Path,
    filter: F,
    log_path: &Path,
    component: &str,
    success_count: &mut usize,
    error_count: &mut usize
) -> Result<()>
where
    F: Fn(&str) -> bool
{
    // Skip if source doesn't exist
    if !source.exists() {
        return Ok(());
    }
    
    // Create target if it doesn't exist
    fs::create_dir_all(target)?;
    
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy().to_string();
        let target_path = target.join(&file_name);
        
        // Apply filter
        if !filter(&file_name_str) {
            continue;
        }
        
        if path.is_dir() {
            // Recursively restore directory
            restore_directory_with_filter(&path, &target_path, &filter, 
                                       log_path, component, success_count, error_count)?;
        } else {
            // Restore file
            match fs::copy(&path, &target_path) {
                Ok(_) => {
                    *success_count += 1;
                    log_recovery_event(log_path, component, "RESTORED", 
                                    &format!("File {:?}", file_name))?;
                },
                Err(e) => {
                    *error_count += 1;
                    log_recovery_event(log_path, component, "ERROR", 
                                    &format!("Failed to restore {:?}: {}", file_name, e))?;
                }
            }
        }
    }
    
    Ok(())
}
