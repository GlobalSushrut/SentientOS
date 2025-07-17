use anyhow::{Result, Context};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

use super::container::{Container, ContainerId, ContainerInfo, ContainerStatus, generate_container_id};
use crate::core::constants;

// In-memory container registry
lazy_static::lazy_static! {
    static ref CONTAINER_REGISTRY: Arc<Mutex<Registry>> = Arc::new(Mutex::new(Registry::new()));
}

/// Container Registry
#[derive(Debug, Default)]
struct Registry {
    /// Map of container ID to container
    containers: HashMap<ContainerId, Container>,
    
    /// Map of container ID to container status
    status: HashMap<ContainerId, ContainerStatus>,
}

impl Registry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
            status: HashMap::new(),
        }
    }
}

/// Registry data for serialization
#[derive(Debug, Serialize, Deserialize)]
struct RegistryData {
    /// Container IDs and their respective paths
    containers: HashMap<ContainerId, String>,
}

/// Initialize the MatrixBox registry
pub fn init() -> Result<()> {
    info!("Initializing MatrixBox registry");
    
    // Create registry directory if it doesn't exist
    let registry_dir = PathBuf::from(constants::ROOT_DIR)
        .join(constants::CONTAINER_DIR)
        .join("registry");
    
    fs::create_dir_all(&registry_dir)
        .context("Failed to create registry directory")?;
    
    // Load registry data if it exists
    let registry_file = registry_dir.join("registry.json");
    if registry_file.exists() {
        load_registry(&registry_file)?;
    }
    
    info!("MatrixBox registry initialized successfully");
    Ok(())
}

/// Shutdown the MatrixBox registry
pub fn shutdown() -> Result<()> {
    info!("Shutting down MatrixBox registry");
    
    // Save registry data
    let registry_dir = PathBuf::from(constants::ROOT_DIR)
        .join(constants::CONTAINER_DIR)
        .join("registry");
    
    let registry_file = registry_dir.join("registry.json");
    save_registry(&registry_file)?;
    
    info!("MatrixBox registry shutdown complete");
    Ok(())
}

/// Load registry data from file
fn load_registry(file_path: &PathBuf) -> Result<()> {
    info!("Loading MatrixBox registry from: {:?}", file_path);
    
    // Read registry file
    let content = fs::read_to_string(file_path)
        .context("Failed to read registry file")?;
    
    let data: RegistryData = serde_json::from_str(&content)
        .context("Failed to parse registry data")?;
    
    // Load containers
    let mut registry = CONTAINER_REGISTRY.lock().unwrap();
    
    for (id, path) in data.containers {
        // Only load containers that still exist
        if PathBuf::from(&path).exists() {
            match super::container::load_container(&path) {
                Ok(mut container) => {
                    container.id = Some(id.clone());
                    registry.containers.insert(id.clone(), container);
                    registry.status.insert(id.clone(), ContainerStatus::Created);
                    info!("Loaded container: {} from registry", id);
                },
                Err(err) => {
                    warn!("Failed to load container {}: {}", id, err);
                }
            }
        }
    }
    
    info!("Loaded {} containers from registry", registry.containers.len());
    Ok(())
}

/// Save registry data to file
fn save_registry(file_path: &PathBuf) -> Result<()> {
    info!("Saving MatrixBox registry to: {:?}", file_path);
    
    // Create registry data
    let registry = CONTAINER_REGISTRY.lock().unwrap();
    
    let mut data = RegistryData {
        containers: HashMap::new(),
    };
    
    for (id, container) in &registry.containers {
        if let Some(path) = &container.path {
            data.containers.insert(id.clone(), path.to_string_lossy().to_string());
        }
    }
    
    // Write registry file
    let content = serde_json::to_string_pretty(&data)
        .context("Failed to serialize registry data")?;
    
    fs::write(file_path, content)
        .context("Failed to write registry file")?;
    
    info!("Saved {} containers to registry", data.containers.len());
    Ok(())
}

/// Register a container in the registry
pub fn register_container(container: &Container) -> Result<ContainerId> {
    let id = generate_container_id();
    info!("Registering container: {} with ID: {}", container.name, id);
    
    let mut registry = CONTAINER_REGISTRY.lock().unwrap();
    
    // Clone the container and set its ID
    let mut container = container.clone();
    container.id = Some(id.clone());
    
    // Add to registry
    registry.containers.insert(id.clone(), container);
    registry.status.insert(id.clone(), ContainerStatus::Created);
    
    info!("Container registered: {}", id);
    Ok(id)
}

/// Unregister a container from the registry
pub fn unregister_container(id: &ContainerId) -> Result<()> {
    info!("Unregistering container: {}", id);
    
    let mut registry = CONTAINER_REGISTRY.lock().unwrap();
    
    if registry.containers.remove(id).is_some() {
        registry.status.remove(id);
        info!("Container unregistered: {}", id);
        Ok(())
    } else {
        anyhow::bail!("Container not found: {}", id);
    }
}

/// Get a container by ID
pub fn get_container(id: &ContainerId) -> Result<Container> {
    let registry = CONTAINER_REGISTRY.lock().unwrap();
    
    if let Some(container) = registry.containers.get(id) {
        Ok(container.clone())
    } else {
        anyhow::bail!("Container not found: {}", id);
    }
}

/// Update a container's status
pub fn update_container_status(id: &ContainerId, status: ContainerStatus) -> Result<()> {
    let mut registry = CONTAINER_REGISTRY.lock().unwrap();
    
    if registry.containers.contains_key(id) {
        registry.status.insert(id.clone(), status);
        info!("Updated container status: {}", id);
        Ok(())
    } else {
        anyhow::bail!("Container not found: {}", id);
    }
}

/// Get a container's status
pub fn get_container_status(id: &ContainerId) -> Result<ContainerStatus> {
    let registry = CONTAINER_REGISTRY.lock().unwrap();
    
    if let Some(status) = registry.status.get(id) {
        Ok(status.clone())
    } else {
        anyhow::bail!("Container status not found: {}", id);
    }
}

/// List all containers
pub fn list_containers() -> Result<Vec<ContainerInfo>> {
    let registry = CONTAINER_REGISTRY.lock().unwrap();
    
    let mut containers = Vec::new();
    
    for (id, container) in &registry.containers {
        let status = registry.status.get(id).cloned().unwrap_or(ContainerStatus::Created);
        
        containers.push(ContainerInfo {
            id: id.clone(),
            name: container.name.clone(),
            status,
            created_at: container.metadata.created_at.clone(),
        });
    }
    
    Ok(containers)
}
