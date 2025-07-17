// SentientOS Healing System
// Provides auto-recovery without reboot and state restoration capabilities

pub mod snapshot;
pub mod recovery;
pub mod verification;

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use blake3;

use crate::core::constants;

/// Initialize the healing system
pub fn init() -> Result<()> {
    info!("Initializing SentientOS healing system");
    
    // Create healing system directories
    let heal_dir = PathBuf::from(constants::ROOT_DIR).join(".heal");
    fs::create_dir_all(&heal_dir)?;
    
    let snapshots_dir = heal_dir.join("snapshots");
    fs::create_dir_all(&snapshots_dir)?;
    
    let recovery_dir = heal_dir.join("recovery");
    fs::create_dir_all(&recovery_dir)?;
    
    let logs_dir = heal_dir.join("logs");
    fs::create_dir_all(&logs_dir)?;
    
    // Initialize components
    snapshot::init()?;
    recovery::init()?;
    verification::init()?;
    
    info!("SentientOS healing system initialized successfully");
    Ok(())
}

/// Shutdown the healing system
pub fn shutdown() -> Result<()> {
    info!("Shutting down SentientOS healing system");
    
    // Take a final snapshot before shutdown
    let snapshot_id = take_snapshot("shutdown")?;
    info!("Created shutdown snapshot: {}", snapshot_id);
    
    // Shutdown components in reverse order
    verification::shutdown()?;
    recovery::shutdown()?;
    snapshot::shutdown()?;
    
    info!("SentientOS healing system shutdown complete");
    Ok(())
}

/// Check system health
pub fn check_health() -> Result<HealthStatus> {
    info!("Checking SentientOS system health");
    
    // Verify critical system components
    let core_status = verification::verify_core_components()?;
    
    // Verify container state
    let container_status = verification::verify_container_state()?;
    
    // Verify ZK contract state
    let zk_status = verification::verify_zk_contract_state()?;
    
    // Determine overall health status
    let status = if core_status && container_status && zk_status {
        HealthStatus::Healthy
    } else if !core_status {
        HealthStatus::Critical
    } else {
        HealthStatus::Degraded
    };
    
    info!("System health status: {:?}", status);
    Ok(status)
}

/// Take a system snapshot
pub fn take_snapshot(reason: &str) -> Result<String> {
    info!("Taking system snapshot: {}", reason);
    
    // Generate snapshot ID
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?
        .as_secs();
    
    let random_suffix = {
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        format!("{:04x}", rng.gen::<u16>())
    };
    
    let snapshot_id = format!("{}-{}-{}", timestamp, reason, random_suffix);
    
    // Create the snapshot
    snapshot::create_snapshot(&snapshot_id, reason)?;
    
    info!("Snapshot created: {}", snapshot_id);
    Ok(snapshot_id)
}

/// Recover from a snapshot
pub fn recover_from_snapshot(snapshot_id: &str) -> Result<()> {
    info!("Recovering from snapshot: {}", snapshot_id);
    
    // Verify the snapshot exists
    let snapshot_path = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("snapshots")
        .join(snapshot_id);
    
    if !snapshot_path.exists() {
        anyhow::bail!("Snapshot not found: {}", snapshot_id);
    }
    
    // Stop running containers
    info!("Stopping running containers for recovery");
    crate::matrixbox::shutdown()?;
    
    // Perform recovery
    recovery::recover_from_snapshot(snapshot_id)?;
    
    // Restart container runtime
    info!("Restarting container runtime");
    crate::matrixbox::init()?;
    
    // Verify recovery
    let health = check_health()?;
    
    if health == HealthStatus::Healthy || health == HealthStatus::Degraded {
        info!("Recovery successful: {:?}", health);
        Ok(())
    } else {
        error!("Recovery failed: {:?}", health);
        anyhow::bail!("Recovery failed: {:?}", health)
    }
}

/// List available snapshots
pub fn list_snapshots() -> Result<Vec<SnapshotInfo>> {
    info!("Listing available snapshots");
    
    snapshot::list_snapshots()
}

/// Get the latest snapshot
pub fn get_latest_snapshot() -> Result<Option<SnapshotInfo>> {
    info!("Getting latest snapshot");
    
    let snapshots = snapshot::list_snapshots()?;
    
    if snapshots.is_empty() {
        info!("No snapshots found");
        return Ok(None);
    }
    
    // Find the latest snapshot by timestamp
    let latest = snapshots.into_iter()
        .max_by_key(|s| s.timestamp);
    
    if let Some(snapshot) = &latest {
        info!("Latest snapshot: {}", snapshot.id);
    } else {
        warn!("Failed to determine latest snapshot");
    }
    
    Ok(latest)
}

/// System health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    
    /// System is degraded but functional
    Degraded,
    
    /// System is in a critical state
    Critical,
}

/// Snapshot information
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    /// Snapshot ID
    pub id: String,
    
    /// Snapshot timestamp (seconds since epoch)
    pub timestamp: u64,
    
    /// Reason for taking the snapshot
    pub reason: String,
    
    /// Path to the snapshot
    pub path: PathBuf,
    
    /// Content hash of the snapshot
    pub hash: String,
}
