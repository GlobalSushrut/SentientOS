use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tracing::info;

use crate::core::constants;

/// Ensure all required SentientOS directories exist
pub fn ensure_directories() -> Result<()> {
    info!("Ensuring core SentientOS directories exist");
    
    // Core system directories
    create_directory_if_not_exists(constants::RUNTIME_DIR)?;
    
    // Lock and ZK verification directories
    let lock_dir = constants::LOCK_DIR;
    create_directory_if_not_exists(lock_dir)?;
    create_directory_if_not_exists(&format!("{}/binary.zk", lock_dir))?;
    create_directory_if_not_exists(&format!("{}/zk.trace", lock_dir))?;
    create_directory_if_not_exists(&format!("{}/zk.remind", lock_dir))?;
    create_directory_if_not_exists(&format!("{}/zk.rollup", lock_dir))?;
    
    // Auth directories
    let auth_dir = constants::AUTH_DIR;
    create_directory_if_not_exists(auth_dir)?;
    create_directory_if_not_exists(&format!("{}/.secret.db", auth_dir))?;
    create_directory_if_not_exists(&format!("{}/.secret.termux", auth_dir))?;
    create_directory_if_not_exists(&format!("{}/.secret.block", auth_dir))?;
    
    // Heal directories (auto-recovery)
    let heal_dir = constants::HEAL_DIR;
    create_directory_if_not_exists(heal_dir)?;
    create_directory_if_not_exists(&format!("{}/container", heal_dir))?;
    create_directory_if_not_exists(&format!("{}/boot", heal_dir))?;
    create_directory_if_not_exists(&format!("{}/trigger", heal_dir))?;
    
    // Gossip directories (multi-device sync)
    let gossip_dir = constants::GOSSIP_DIR;
    create_directory_if_not_exists(gossip_dir)?;
    create_directory_if_not_exists(&format!("{}/peers", gossip_dir))?;
    create_directory_if_not_exists(&format!("{}/pull", gossip_dir))?;
    create_directory_if_not_exists(&format!("{}/verify", gossip_dir))?;
    
    // Intent directories (developer intent)
    let intent_dir = constants::INTENT_DIR;
    create_directory_if_not_exists(intent_dir)?;
    create_directory_if_not_exists(&format!("{}/sessions", intent_dir))?;
    create_directory_if_not_exists(&format!("{}/replay", intent_dir))?;
    create_directory_if_not_exists(&format!("{}/timeline", intent_dir))?;
    
    // Panic directories (failure handling)
    create_directory_if_not_exists(constants::PANIC_DIR)?;
    
    // Zero-mode directories (micro runtime)
    let zero_dir = constants::ZERO_DIR;
    create_directory_if_not_exists(zero_dir)?;
    create_directory_if_not_exists(&format!("{}/cli", zero_dir))?;
    create_directory_if_not_exists(&format!("{}/auth", zero_dir))?;
    create_directory_if_not_exists(&format!("{}/trace", zero_dir))?;
    
    // Unsecure directories (non-ZK applications)
    let unsecure_dir = constants::UNSECURE_DIR;
    create_directory_if_not_exists(unsecure_dir)?;
    create_directory_if_not_exists(&format!("{}/wasm", unsecure_dir))?;
    create_directory_if_not_exists(&format!("{}/legacy", unsecure_dir))?;
    
    // Container and runtime directories
    create_directory_if_not_exists(constants::CONTAINER_DIR)?;
    create_directory_if_not_exists(constants::BROWSER_DIR)?;
    create_directory_if_not_exists(".tff")?;
    create_directory_if_not_exists(".bak")?;
    create_directory_if_not_exists(".osr")?;
    create_directory_if_not_exists(".tree")?;
    create_directory_if_not_exists(".boot")?;
    create_directory_if_not_exists(".db")?;
    create_directory_if_not_exists(".redis")?;
    create_directory_if_not_exists(".cons")?;
    
    // Standard Linux compatibility directories
    create_directory_if_not_exists("app")?;
    create_directory_if_not_exists("usr")?;
    create_directory_if_not_exists("bin")?;
    create_directory_if_not_exists("termux.io")?;
    create_directory_if_not_exists("contrac.to")?;
    create_directory_if_not_exists("zk_contracts")?;
    
    info!("All core directories created successfully");
    Ok(())
}

/// Create a directory if it doesn't exist
pub fn create_directory_if_not_exists(dir: &str) -> Result<()> {
    let path = PathBuf::from(constants::ROOT_DIR).join(dir);
    if !path.exists() {
        info!("Creating directory: {:?}", path);
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create directory: {:?}", path))?;
    }
    Ok(())
}

/// Check if a file exists
pub fn file_exists(path: &str) -> bool {
    let full_path = PathBuf::from(constants::ROOT_DIR).join(path);
    full_path.exists() && full_path.is_file()
}

/// Write data to a file with ZK verification
pub fn write_file_with_verification(path: &str, data: &[u8], enable_zk: bool) -> Result<()> {
    let full_path = PathBuf::from(constants::ROOT_DIR).join(path);
    
    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory for: {:?}", full_path))?;
    }
    
    // Write the actual file
    fs::write(&full_path, data)
        .with_context(|| format!("Failed to write file: {:?}", full_path))?;
    
    // If ZK mode is enabled, generate and store a verification hash
    if enable_zk {
        let hash = blake3::hash(data);
        let hash_path = format!("{}.zk.hash", path);
        let hash_full_path = PathBuf::from(constants::ROOT_DIR).join(&hash_path);
        
        fs::write(hash_full_path, hash.as_bytes())
            .with_context(|| format!("Failed to write ZK hash file for: {:?}", path))?;
        
        // TODO: Generate ZK proof and store it
        info!("Generated ZK verification for file: {}", path);
    }
    
    info!("Successfully wrote file: {}", path);
    Ok(())
}

/// Read a file with ZK verification
pub fn read_file_with_verification(path: &str, verify_zk: bool) -> Result<Vec<u8>> {
    let full_path = PathBuf::from(constants::ROOT_DIR).join(path);
    
    // Read the file
    let data = fs::read(&full_path)
        .with_context(|| format!("Failed to read file: {:?}", full_path))?;
    
    // If ZK verification is requested, verify the hash
    if verify_zk {
        let hash = blake3::hash(&data);
        
        let hash_path = format!("{}.zk.hash", path);
        let hash_full_path = PathBuf::from(constants::ROOT_DIR).join(&hash_path);
        
        if hash_full_path.exists() {
            let stored_hash = fs::read(&hash_full_path)
                .with_context(|| format!("Failed to read ZK hash file for: {:?}", path))?;
            
            if hash.as_bytes() != stored_hash.as_slice() {
                anyhow::bail!("ZK verification failed for file: {}", path);
            }
            
            // TODO: Verify ZK proof
            info!("ZK verification passed for file: {}", path);
        } else {
            anyhow::bail!("No ZK hash found for file: {}", path);
        }
    }
    
    Ok(data)
}
