// SentientOS MatrixBox Container Runtime
// A native Rust-WASM tree runtime with ZK-proofed memory trees

pub mod container;
pub mod runtime;
pub mod registry;
pub mod wasm;
pub mod tso;

use anyhow::Result;
use tracing::{info, warn};
use std::path::PathBuf;

use crate::core::constants;

/// Initialize the MatrixBox container runtime
pub fn init() -> Result<()> {
    info!("Initializing MatrixBox container runtime");
    
    // Create container directory if it doesn't exist
    let container_dir = PathBuf::from(constants::ROOT_DIR).join(constants::CONTAINER_DIR);
    std::fs::create_dir_all(&container_dir)?;
    
    // Create TSO archive directory
    let tso_dir = PathBuf::from(constants::ROOT_DIR).join(".matrixbox").join("tso");
    std::fs::create_dir_all(&tso_dir)?;
    
    // Initialize container registry
    registry::init()?;
    
    // Initialize WASM runtime
    wasm::init()?;
    
    // Initialize container runtime
    runtime::init()?;
    
    info!("MatrixBox container runtime initialized successfully");
    Ok(())
}

/// Shutdown the MatrixBox container runtime
pub fn shutdown() -> Result<()> {
    info!("Shutting down MatrixBox container runtime");
    
    // Shutdown components in reverse order
    runtime::shutdown()?;
    wasm::shutdown()?;
    registry::shutdown()?;
    
    info!("MatrixBox container runtime shutdown complete");
    Ok(())
}

/// Run a MatrixBox container
pub fn run_container(container_path: &str) -> Result<container::ContainerId> {
    info!("Running MatrixBox container: {}", container_path);
    
    // Check if this is a TSO archive
    let path = PathBuf::from(container_path);
    let is_tso = path.extension()
        .map(|ext| ext == "tso")
        .unwrap_or(false);
    
    let container = if is_tso {
        info!("Loading TSO container archive: {}", container_path);
        
        // Extract TSO to temporary directory
        let temp_dir = PathBuf::from(constants::ROOT_DIR)
            .join(".matrixbox")
            .join("extracted")
            .join(format!("{}", chrono::Utc::now().timestamp()));
        
        std::fs::create_dir_all(&temp_dir)?;
        tso::extract_tso_archive(&path, &temp_dir)?
    } else {
        // Load the container normally
        container::load_container(container_path)?
    };
    
    // Register the container
    let id = registry::register_container(&container)?;
    
    // Start the container with WASM runtime
    let args = Vec::new();
    wasm::run_container(&container, &args.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;
    
    info!("MatrixBox container started: {}", id);
    Ok(id)
}

/// Stop a running MatrixBox container
pub fn stop_container(id: &container::ContainerId) -> Result<()> {
    info!("Stopping MatrixBox container: {}", id);
    
    // Stop the WASM instance
    wasm::stop_container(id)?;
    
    // Stop the container runtime
    runtime::stop_container(id)?;
    
    info!("MatrixBox container stopped: {}", id);
    Ok(())
}

/// List all running MatrixBox containers
pub fn list_containers() -> Result<Vec<container::ContainerInfo>> {
    info!("Listing all running MatrixBox containers");
    
    // Get containers from registry
    let containers = registry::list_containers()?;
    
    info!("Found {} running MatrixBox containers", containers.len());
    Ok(containers)
}

/// Remove a MatrixBox container
pub fn remove_container(id: &container::ContainerId) -> Result<()> {
    info!("Removing MatrixBox container: {}", id);
    
    // Ensure container is stopped
    if runtime::is_container_running(id)? {
        runtime::stop_container(id)?;
    }
    
    // Unregister the container
    registry::unregister_container(id)?;
    
    info!("MatrixBox container removed: {}", id);
    Ok(())
}
