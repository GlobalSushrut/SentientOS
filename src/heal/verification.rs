use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use std::path::{Path, PathBuf};
use std::fs;
use blake3;

use crate::core::constants;

/// Initialize the verification system
pub fn init() -> Result<()> {
    info!("Initializing verification system");
    
    // Ensure verification directory exists
    let verify_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("verification");
    
    fs::create_dir_all(&verify_dir)
        .context("Failed to create verification directory")?;
    
    info!("Verification system initialized");
    Ok(())
}

/// Shutdown the verification system
pub fn shutdown() -> Result<()> {
    info!("Shutting down verification system");
    
    // Nothing specific to shut down
    
    info!("Verification system shutdown complete");
    Ok(())
}

/// Verify core system components
pub fn verify_core_components() -> Result<bool> {
    info!("Verifying core system components");
    
    let mut all_valid = true;
    
    // Check if core directories exist
    all_valid &= verify_directory_exists(constants::CORE_DIR)?;
    all_valid &= verify_directory_exists(constants::ZK_DIR)?;
    all_valid &= verify_directory_exists(constants::CONTAINER_DIR)?;
    
    // Check core configuration files
    let core_config = PathBuf::from(constants::ROOT_DIR)
        .join(constants::CORE_DIR)
        .join("config.yaml");
    
    if !core_config.exists() {
        warn!("Core configuration file missing: {:?}", core_config);
        all_valid = false;
    }
    
    // Verify integrity of key files using stored hashes
    all_valid &= verify_file_integrity(constants::CORE_DIR, "config.yaml")?;
    all_valid &= verify_file_integrity(constants::ZK_DIR, "registry.json")?;
    all_valid &= verify_file_integrity(constants::CONTAINER_DIR, "registry.json")?;
    
    info!("Core component verification complete: {}", all_valid);
    Ok(all_valid)
}

/// Verify container state
pub fn verify_container_state() -> Result<bool> {
    info!("Verifying container state");
    
    let mut all_valid = true;
    
    // Check container registry
    let registry_path = PathBuf::from(constants::ROOT_DIR)
        .join(constants::CONTAINER_DIR)
        .join("registry.json");
    
    if registry_path.exists() {
        // Parse the registry file
        let registry_data = fs::read_to_string(&registry_path)
            .with_context(|| format!("Failed to read container registry: {:?}", registry_path))?;
        
        // Simple JSON validation
        match serde_json::from_str::<serde_json::Value>(&registry_data) {
            Ok(_) => {
                debug!("Container registry is valid JSON");
            },
            Err(e) => {
                warn!("Container registry is corrupted: {}", e);
                all_valid = false;
            }
        }
    } else {
        warn!("Container registry file not found: {:?}", registry_path);
        all_valid = false;
    }
    
    // Check container directories
    let containers_dir = PathBuf::from(constants::ROOT_DIR)
        .join(constants::CONTAINER_DIR)
        .join("instances");
    
    if !containers_dir.exists() {
        warn!("Container instances directory missing: {:?}", containers_dir);
        all_valid = false;
    }
    
    info!("Container state verification complete: {}", all_valid);
    Ok(all_valid)
}

/// Verify ZK contract state
pub fn verify_zk_contract_state() -> Result<bool> {
    info!("Verifying ZK contract state");
    
    let mut all_valid = true;
    
    // Check ZK contracts directory
    let contracts_dir = PathBuf::from(constants::ROOT_DIR)
        .join(constants::ZK_DIR)
        .join("contracts");
    
    if !contracts_dir.exists() {
        warn!("ZK contracts directory missing: {:?}", contracts_dir);
        all_valid = false;
    } else {
        // Verify each contract file
        if let Ok(entries) = fs::read_dir(&contracts_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
                    // Basic YAML validation
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            match serde_yaml::from_str::<serde_yaml::Value>(&content) {
                                Ok(_) => {
                                    debug!("Contract is valid YAML: {:?}", path);
                                },
                                Err(e) => {
                                    warn!("Contract is corrupted YAML: {:?}, error: {}", path, e);
                                    all_valid = false;
                                }
                            }
                        },
                        Err(e) => {
                            warn!("Failed to read contract file: {:?}, error: {}", path, e);
                            all_valid = false;
                        }
                    }
                }
            }
        }
    }
    
    // Check ZK verification keys
    let keys_dir = PathBuf::from(constants::ROOT_DIR)
        .join(constants::ZK_DIR)
        .join("keys");
    
    if !keys_dir.exists() {
        warn!("ZK keys directory missing: {:?}", keys_dir);
        // Not critical, just warn
    }
    
    info!("ZK contract state verification complete: {}", all_valid);
    Ok(all_valid)
}

/// Verify a directory exists
fn verify_directory_exists(dir_name: &str) -> Result<bool> {
    let dir_path = PathBuf::from(constants::ROOT_DIR).join(dir_name);
    
    if dir_path.exists() && dir_path.is_dir() {
        debug!("Directory exists: {:?}", dir_path);
        Ok(true)
    } else {
        warn!("Directory missing: {:?}", dir_path);
        Ok(false)
    }
}

/// Verify file integrity using stored hash
fn verify_file_integrity(dir_name: &str, file_name: &str) -> Result<bool> {
    let file_path = PathBuf::from(constants::ROOT_DIR)
        .join(dir_name)
        .join(file_name);
    
    let hash_path = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("verification")
        .join(format!("{}_{}.hash", dir_name.replace("/", "_"), file_name));
    
    if !file_path.exists() {
        warn!("File not found for integrity check: {:?}", file_path);
        return Ok(false);
    }
    
    if !hash_path.exists() {
        // No stored hash to compare against, create one
        debug!("No stored hash found for {:?}, creating one", file_path);
        let hash = compute_file_hash(&file_path)?;
        
        // Ensure parent directory exists
        if let Some(parent) = hash_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Store the hash
        fs::write(&hash_path, hash.as_bytes())?;
        return Ok(true);
    }
    
    // Compare stored hash with current hash
    let stored_hash = fs::read_to_string(&hash_path)
        .with_context(|| format!("Failed to read stored hash: {:?}", hash_path))?;
    
    let current_hash = compute_file_hash(&file_path)?;
    
    if stored_hash == current_hash {
        debug!("File integrity verified: {:?}", file_path);
        Ok(true)
    } else {
        warn!("File integrity check failed: {:?}", file_path);
        warn!("  Expected: {}", stored_hash);
        warn!("  Actual: {}", current_hash);
        Ok(false)
    }
}

/// Compute hash for a file
fn compute_file_hash(file_path: &Path) -> Result<String> {
    let content = fs::read(file_path)
        .with_context(|| format!("Failed to read file for hashing: {:?}", file_path))?;
    
    let hash = blake3::hash(&content);
    Ok(hash.to_hex().to_string())
}

/// Update stored hash for a file
pub fn update_file_hash(dir_name: &str, file_name: &str) -> Result<()> {
    let file_path = PathBuf::from(constants::ROOT_DIR)
        .join(dir_name)
        .join(file_name);
    
    let hash_path = PathBuf::from(constants::ROOT_DIR)
        .join(".heal")
        .join("verification")
        .join(format!("{}_{}.hash", dir_name.replace("/", "_"), file_name));
    
    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {:?}", file_path));
    }
    
    // Compute and store hash
    let hash = compute_file_hash(&file_path)?;
    
    // Ensure parent directory exists
    if let Some(parent) = hash_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&hash_path, hash.as_bytes())?;
    debug!("Updated hash for file: {:?}", file_path);
    
    Ok(())
}
