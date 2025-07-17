// SentientOS Panic System
// Handles failure trap & recovery

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use serde_json;

use crate::core::constants;
use crate::heal;

/// Initialize the panic system
pub fn init() -> Result<()> {
    info!("Initializing SentientOS panic system");
    
    // Create panic system directories
    let panic_dir = PathBuf::from(constants::ROOT_DIR).join(".panic");
    fs::create_dir_all(&panic_dir)?;
    
    // Create initial fallback.zk file with last known good state
    let fallback_path = panic_dir.join("fallback.zk");
    if !fallback_path.exists() {
        let initial_state = FallbackState {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            status: "initial".to_string(),
            heal_snapshot_id: None,
        };
        
        let fallback_content = serde_json::to_string_pretty(&initial_state)?;
        fs::write(&fallback_path, fallback_content)?;
    }
    
    // Create trace recovery directory
    let trace_dir = panic_dir.join("trace.recover");
    fs::create_dir_all(&trace_dir)?;
    
    // Create crash reporting directory
    let report_dir = panic_dir.join("log.send");
    fs::create_dir_all(&report_dir)?;
    
    info!("SentientOS panic system initialized successfully");
    Ok(())
}

/// Shutdown the panic system
pub fn shutdown() -> Result<()> {
    info!("Shutting down SentientOS panic system");
    
    // Update fallback.zk with current known good state
    update_fallback_state("shutdown", None)?;
    
    info!("SentientOS panic system shutdown complete");
    Ok(())
}

/// Record a panic event
pub fn record_panic(reason: &str, details: &str) -> Result<()> {
    error!("SYSTEM PANIC: {}", reason);
    
    // Record panic timestamp
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    
    // Create panic record
    let panic_record = PanicRecord {
        timestamp,
        reason: reason.to_string(),
        details: details.to_string(),
    };
    
    // Save panic record
    let panic_dir = PathBuf::from(constants::ROOT_DIR).join(".panic");
    let panic_file = panic_dir.join(format!("panic-{}.json", timestamp));
    let panic_content = serde_json::to_string_pretty(&panic_record)?;
    fs::write(&panic_file, panic_content)?;
    
    // Update current panic status
    let status_file = panic_dir.join("status.json");
    let status = PanicStatus {
        active: true,
        timestamp,
        reason: reason.to_string(),
        recovery_attempted: false,
    };
    let status_content = serde_json::to_string_pretty(&status)?;
    fs::write(&status_file, status_content)?;
    
    // Take a snapshot for potential recovery
    match heal::take_snapshot(&format!("panic-{}", reason)) {
        Ok(snapshot_id) => {
            info!("Created panic snapshot: {}", snapshot_id);
            update_fallback_state("panic", Some(&snapshot_id))?;
        }
        Err(e) => {
            error!("Failed to create panic snapshot: {:?}", e);
        }
    }
    
    Ok(())
}

/// Recover from a panic state
pub fn recover() -> Result<()> {
    info!("Recovering from panic state");
    
    // Check if system is actually in a panic state
    let panic_dir = PathBuf::from(constants::ROOT_DIR).join(".panic");
    let status_file = panic_dir.join("status.json");
    
    if !status_file.exists() {
        info!("No active panic state found");
        return Ok(());
    }
    
    // Read panic status
    let status_content = fs::read_to_string(&status_file)?;
    let mut status: PanicStatus = serde_json::from_str(&status_content)?;
    
    if !status.active {
        info!("No active panic state found");
        return Ok(());
    }
    
    // Get fallback state
    let fallback_path = panic_dir.join("fallback.zk");
    let fallback_content = fs::read_to_string(&fallback_path)?;
    let fallback: FallbackState = serde_json::from_str(&fallback_content)?;
    
    // Attempt recovery from snapshot if available
    if let Some(snapshot_id) = &fallback.heal_snapshot_id {
        info!("Attempting recovery from snapshot: {}", snapshot_id);
        
        match heal::recover_from_snapshot(snapshot_id) {
            Ok(()) => {
                info!("Successfully recovered from snapshot");
                
                // Update panic status
                status.recovery_attempted = true;
                status.active = false;
                let status_content = serde_json::to_string_pretty(&status)?;
                fs::write(&status_file, status_content)?;
                
                return Ok(());
            }
            Err(e) => {
                error!("Failed to recover from snapshot: {:?}", e);
                // Fall through to manual recovery
            }
        }
    }
    
    // If we reached here, snapshot recovery failed or wasn't available
    warn!("No valid recovery snapshot available. Manual recovery required.");
    
    // Update panic status
    status.recovery_attempted = true;
    let status_content = serde_json::to_string_pretty(&status)?;
    fs::write(&status_file, status_content)?;
    
    Ok(())
}

/// Generate a crash report
pub fn generate_report(output_path: &str) -> Result<()> {
    info!("Generating crash report: {}", output_path);
    
    // Get panic directory
    let panic_dir = PathBuf::from(constants::ROOT_DIR).join(".panic");
    
    // Collect all panic records
    let mut panic_records = Vec::new();
    for entry in fs::read_dir(&panic_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("panic-") && file_name.ends_with(".json") {
                    let content = fs::read_to_string(&path)?;
                    match serde_json::from_str::<PanicRecord>(&content) {
                        Ok(record) => panic_records.push(record),
                        Err(_) => continue, // Skip invalid records
                    }
                }
            }
        }
    }
    
    // Get system information
    let system_info = SystemInfo {
        os_version: "SentientOS 1.0".to_string(),
        boot_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - 3600, // Fake boot time
        zk_enabled: true,
        containers_running: 0, // This would be fetched from matrixbox
    };
    
    // Create crash report
    let crash_report = CrashReport {
        generated_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        system_info,
        panic_records,
    };
    
    // Write crash report
    let report_content = serde_json::to_string_pretty(&crash_report)?;
    fs::write(output_path, report_content)?;
    
    info!("Crash report generated successfully: {}", output_path);
    Ok(())
}

/// Update fallback state
fn update_fallback_state(status: &str, snapshot_id: Option<&str>) -> Result<()> {
    let panic_dir = PathBuf::from(constants::ROOT_DIR).join(".panic");
    let fallback_path = panic_dir.join("fallback.zk");
    
    let fallback_state = FallbackState {
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        status: status.to_string(),
        heal_snapshot_id: snapshot_id.map(String::from),
    };
    
    let fallback_content = serde_json::to_string_pretty(&fallback_state)?;
    fs::write(&fallback_path, fallback_content)?;
    
    Ok(())
}

/// Fallback state
#[derive(Debug, Serialize, Deserialize)]
struct FallbackState {
    /// Timestamp when the state was recorded
    timestamp: u64,
    
    /// Status of the system
    status: String,
    
    /// ID of heal snapshot to use for recovery
    heal_snapshot_id: Option<String>,
}

/// Panic record
#[derive(Debug, Serialize, Deserialize)]
struct PanicRecord {
    /// Timestamp when the panic occurred
    timestamp: u64,
    
    /// Reason for the panic
    reason: String,
    
    /// Detailed information about the panic
    details: String,
}

/// Panic status
#[derive(Debug, Serialize, Deserialize)]
struct PanicStatus {
    /// Whether a panic is currently active
    active: bool,
    
    /// Timestamp of the panic
    timestamp: u64,
    
    /// Reason for the panic
    reason: String,
    
    /// Whether recovery has been attempted
    recovery_attempted: bool,
}

/// System information
#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    /// OS version
    os_version: String,
    
    /// Boot time (seconds since epoch)
    boot_time: u64,
    
    /// Whether ZK is enabled
    zk_enabled: bool,
    
    /// Number of running containers
    containers_running: usize,
}

/// Crash report
#[derive(Debug, Serialize, Deserialize)]
struct CrashReport {
    /// When the report was generated
    generated_at: u64,
    
    /// System information
    system_info: SystemInfo,
    
    /// Panic records
    panic_records: Vec<PanicRecord>,
}
