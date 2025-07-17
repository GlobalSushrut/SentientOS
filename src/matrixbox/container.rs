use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use std::fs;

use crate::core::constants;

/// Container ID type
pub type ContainerId = String;

/// MatrixBox TSO Container structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    /// Container ID (generated at runtime)
    #[serde(skip)]
    pub id: Option<ContainerId>,
    
    /// Container name
    pub name: String,
    
    /// Container version
    pub version: String,
    
    /// Container author
    pub author: Option<String>,
    
    /// Container description
    pub description: Option<String>,
    
    /// Path to the container on disk
    #[serde(skip)]
    pub path: Option<PathBuf>,
    
    /// Container metadata from meta.yaml
    pub metadata: ContainerMetadata,
    
    /// Container permissions
    pub permissions: ContainerPermissions,
}

/// Container metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMetadata {
    /// Container creation time
    pub created_at: String,
    
    /// Container WASM entrypoint
    pub entrypoint: String,
    
    /// Container environment variables
    pub environment: Vec<String>,
    
    /// Container dependencies
    pub dependencies: Vec<String>,
    
    /// Container hash tree root
    pub hash_tree_root: String,
}

/// Container permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPermissions {
    /// Filesystem access permissions
    pub filesystem: Vec<String>,
    
    /// Network access permissions
    pub network: NetworkPermissions,
    
    /// Memory limit in bytes
    pub memory_limit: u64,
    
    /// CPU limit (percentage)
    pub cpu_limit: u8,
}

/// Network permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermissions {
    /// Outbound network access
    pub outbound: bool,
    
    /// Inbound network access
    pub inbound: bool,
    
    /// Allowed hosts
    pub allowed_hosts: Vec<String>,
}

/// Container information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    /// Container ID
    pub id: ContainerId,
    
    /// Container name
    pub name: String,
    
    /// Container status
    pub status: ContainerStatus,
    
    /// Container creation time
    pub created_at: String,
}

/// Container status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContainerStatus {
    /// Container is created but not running
    Created,
    
    /// Container is running
    Running,
    
    /// Container is paused
    Paused,
    
    /// Container has exited
    Exited(i32), // Exit code
    
    /// Container has failed
    Failed(String), // Error message
}

/// Load a MatrixBox container from disk
pub fn load_container(container_path: &str) -> Result<Container> {
    info!("Loading MatrixBox container from: {}", container_path);
    
    let path = PathBuf::from(container_path);
    
    // Ensure the path exists and is a directory
    if !path.exists() {
        anyhow::bail!("Container path does not exist: {:?}", path);
    }
    
    if !path.is_dir() {
        anyhow::bail!("Container path is not a directory: {:?}", path);
    }
    
    // Check for required TSO container files
    let meta_path = path.join("meta.yaml");
    let wasm_path = path.join("main.wasm");
    let permissions_path = path.join("permissions.zky");
    
    if !meta_path.exists() {
        anyhow::bail!("Container meta.yaml not found: {:?}", meta_path);
    }
    
    if !wasm_path.exists() {
        anyhow::bail!("Container main.wasm not found: {:?}", wasm_path);
    }
    
    if !permissions_path.exists() {
        anyhow::bail!("Container permissions.zky not found: {:?}", permissions_path);
    }
    
    // Load and parse container metadata
    let meta_content = fs::read_to_string(&meta_path)
        .with_context(|| format!("Failed to read meta.yaml: {:?}", meta_path))?;
    
    let metadata: ContainerMetadata = serde_yaml::from_str(&meta_content)
        .with_context(|| format!("Failed to parse meta.yaml: {:?}", meta_path))?;
    
    // Load and parse container permissions
    let permissions_content = fs::read_to_string(&permissions_path)
        .with_context(|| format!("Failed to read permissions.zky: {:?}", permissions_path))?;
    
    let permissions: ContainerPermissions = serde_yaml::from_str(&permissions_content)
        .with_context(|| format!("Failed to parse permissions.zky: {:?}", permissions_path))?;
    
    // Extract container name and version from meta.yaml
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Create the container
    let container = Container {
        id: None,
        name,
        version: "1.0.0".to_string(), // Default version
        author: None,
        description: None,
        path: Some(path),
        metadata,
        permissions,
    };
    
    info!("Successfully loaded MatrixBox container: {}", container.name);
    Ok(container)
}

/// Create a new MatrixBox container
pub fn create_container(name: &str, entrypoint: &str) -> Result<Container> {
    info!("Creating new MatrixBox container: {}", name);
    
    // Generate container directory path
    let container_dir = PathBuf::from(constants::ROOT_DIR)
        .join(constants::CONTAINER_DIR)
        .join(name);
    
    // Ensure container directory doesn't exist already
    if container_dir.exists() {
        anyhow::bail!("Container already exists: {:?}", container_dir);
    }
    
    // Create container directory
    fs::create_dir_all(&container_dir)
        .with_context(|| format!("Failed to create container directory: {:?}", container_dir))?;
    
    // Create basic container metadata
    let metadata = ContainerMetadata {
        created_at: chrono::Utc::now().to_rfc3339(),
        entrypoint: entrypoint.to_string(),
        environment: Vec::new(),
        dependencies: Vec::new(),
        hash_tree_root: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    };
    
    // Create default container permissions
    let permissions = ContainerPermissions {
        filesystem: vec![
            format!(".container/{}", name),
        ],
        network: NetworkPermissions {
            outbound: false,
            inbound: false,
            allowed_hosts: Vec::new(),
        },
        memory_limit: 1024 * 1024 * 100, // 100MB
        cpu_limit: 50, // 50% CPU
    };
    
    // Write container files
    let meta_yaml = serde_yaml::to_string(&metadata)
        .context("Failed to serialize container metadata")?;
    
    let permissions_yaml = serde_yaml::to_string(&permissions)
        .context("Failed to serialize container permissions")?;
    
    fs::write(container_dir.join("meta.yaml"), meta_yaml)
        .context("Failed to write meta.yaml")?;
    
    fs::write(container_dir.join("permissions.zky"), permissions_yaml)
        .context("Failed to write permissions.zky")?;
    
    // Create empty main.wasm file
    fs::write(container_dir.join("main.wasm"), [])
        .context("Failed to write empty main.wasm")?;
    
    // Create the container
    let container = Container {
        id: None,
        name: name.to_string(),
        version: "1.0.0".to_string(),
        author: None,
        description: None,
        path: Some(container_dir),
        metadata,
        permissions,
    };
    
    info!("Successfully created MatrixBox container: {}", name);
    Ok(container)
}

/// Save container back to disk
pub fn save_container(container: &Container) -> Result<()> {
    let path = container.path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Container has no path"))?;
    
    info!("Saving MatrixBox container: {} to {:?}", container.name, path);
    
    // Write metadata
    let meta_yaml = serde_yaml::to_string(&container.metadata)
        .context("Failed to serialize container metadata")?;
    
    fs::write(path.join("meta.yaml"), meta_yaml)
        .context("Failed to write meta.yaml")?;
    
    // Write permissions
    let permissions_yaml = serde_yaml::to_string(&container.permissions)
        .context("Failed to serialize container permissions")?;
    
    fs::write(path.join("permissions.zky"), permissions_yaml)
        .context("Failed to write permissions.zky")?;
    
    info!("Successfully saved MatrixBox container: {}", container.name);
    Ok(())
}

/// Generate a new container ID
pub fn generate_container_id() -> ContainerId {
    use rand::{thread_rng, Rng};
    
    let mut rng = thread_rng();
    format!("{:016x}", rng.gen::<u64>())
}

/// Example TSO container structure
pub fn example_container_files() -> Vec<(String, String)> {
    vec![
        (
            "meta.yaml".to_string(),
            r#"# MatrixBox Container Metadata
created_at: '2025-07-16T23:30:00Z'
entrypoint: main
environment:
  - RUST_LOG=info
  - SENTIENT_MODE=standard
dependencies:
  - std.wasm
hash_tree_root: '0000000000000000000000000000000000000000000000000000000000000000'
"#.to_string()
        ),
        (
            "permissions.zky".to_string(),
            r#"# MatrixBox Container Permissions
filesystem:
  - .container/example
  - .runtime/logs
network:
  outbound: true
  inbound: false
  allowed_hosts:
    - api.example.com
memory_limit: 104857600  # 100MB
cpu_limit: 50  # 50% CPU
"#.to_string()
        ),
        (
            "main.wasm".to_string(),
            "// Binary WASM content would go here".to_string()
        ),
    ]
}
