// SentientOS MatrixBox WASM Runtime
// Handles execution of WebAssembly modules in containers

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use wasmer::{Instance, Module, Store, Value, Function, imports};
use wasmer_wasi::{WasiState, WasiEnv};
use serde::{Serialize, Deserialize};

use crate::core::constants;
use crate::zk;

use super::container::{Container, ContainerStatus, ContainerId};

// Global registry for running WASM instances
lazy_static::lazy_static! {
    static ref WASM_INSTANCES: Arc<Mutex<HashMap<ContainerId, WasmInstanceInfo>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

/// Initialize the WASM runtime
pub fn init() -> Result<()> {
    info!("Initializing MatrixBox WASM runtime");
    
    // Create necessary directories
    let wasm_dir = PathBuf::from(constants::ROOT_DIR).join(".matrixbox").join("wasm");
    fs::create_dir_all(&wasm_dir)?;
    
    // Clear any stale instance info
    let mut instances = WASM_INSTANCES.lock().unwrap();
    instances.clear();
    
    info!("MatrixBox WASM runtime initialized successfully");
    Ok(())
}

/// Shutdown the WASM runtime
pub fn shutdown() -> Result<()> {
    info!("Shutting down MatrixBox WASM runtime");
    
    // Stop all running instances
    let mut instances = WASM_INSTANCES.lock().unwrap();
    for (container_id, _) in instances.drain() {
        info!("Stopping WASM instance for container: {}", container_id);
    }
    
    info!("MatrixBox WASM runtime shutdown complete");
    Ok(())
}

/// Run a container's WASM module
pub fn run_container(container: &Container, args: &[&str]) -> Result<ContainerId> {
    let container_id = container.id.clone()
        .unwrap_or_else(|| super::container::generate_container_id());
    
    info!("Running WASM module for container: {} (ID: {})", 
          container.name, container_id);
    
    let container_path = container.path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Container has no path"))?;
    
    let wasm_path = container_path.join("main.wasm");
    
    // Ensure the WASM file exists
    if !wasm_path.exists() {
        return Err(anyhow::anyhow!("WASM file not found: {:?}", wasm_path));
    }
    
    // Verify container permissions with ZK contract
    let zk_contract_path = container_path.join("permissions.zky");
    debug!("Loading ZK contract for container permissions: {:?}", zk_contract_path);
    
    let contract = zk::load_contract(zk_contract_path.to_str().unwrap())?;
    let verified = zk::verify_contract(&contract)?;
    
    if !verified {
        return Err(anyhow::anyhow!("Container permissions verification failed"));
    }
    
    // Read the WASM module
    let wasm_bytes = fs::read(&wasm_path)
        .with_context(|| format!("Failed to read WASM file: {:?}", wasm_path))?;
    
    debug!("Loaded WASM module: {} bytes", wasm_bytes.len());
    
    // Create a wasmer store
    let mut store = Store::default();
    
    // Compile the WASM module
    let module = Module::new(&store, &wasm_bytes)
        .with_context(|| "Failed to compile WASM module")?;
    
    // Create WASI environment
    let mut wasi_env_builder = WasiState::new(container.name.clone());
    
    // Add container-specific environment variables
    for env_var in &container.metadata.environment {
        if let Some((key, value)) = env_var.split_once('=') {
            wasi_env_builder = wasi_env_builder.env(key, value);
        }
    }
    
    // Apply filesystem permissions
    for path in &container.permissions.filesystem {
        let fs_path = PathBuf::from(constants::ROOT_DIR).join(path);
        if fs_path.exists() {
            wasi_env_builder = wasi_env_builder.preopen_dir(fs_path, path)?;
        } else {
            warn!("Container requested access to non-existent path: {}", path);
        }
    }
    
    // Capture command line arguments
    for arg in args {
        wasi_env_builder = wasi_env_builder.arg(arg);
    }
    
    let wasi_env = wasi_env_builder.finalize()?;
    
    // Get import object from WASI
    let import_object = wasi_env.import_object(&mut store, &module)?;
    
    // Instantiate the module with imports
    let instance = Instance::new(&mut store, &module, &import_object)
        .with_context(|| "Failed to instantiate WASM module")?;
    
    // Get the WASM memory export
    let memory = instance.exports.get_memory("memory")?;
    
    // Record instance info
    let instance_info = WasmInstanceInfo {
        container_id: container_id.clone(),
        container_name: container.name.clone(),
        start_time: chrono::Utc::now().to_rfc3339(),
        status: WasmInstanceStatus::Running,
        memory_usage: memory.size().bytes().0 as u64,
    };
    
    // Store the instance
    let mut instances = WASM_INSTANCES.lock().unwrap();
    instances.insert(container_id.clone(), instance_info);
    
    // Call the _start function (WASI entry point)
    if let Ok(start) = instance.exports.get_function("_start") {
        debug!("Calling _start function");
        match start.call(&mut store, &[]) {
            Ok(_) => {
                info!("WASM instance started successfully: {}", container_id);
            },
            Err(e) => {
                error!("Error in WASM execution: {}", e);
                // Update status to failed
                if let Some(instance_info) = instances.get_mut(&container_id) {
                    instance_info.status = WasmInstanceStatus::Failed(e.to_string());
                }
                return Err(anyhow::anyhow!("WASM execution failed: {}", e));
            }
        }
    } else {
        // Try main function as fallback
        if let Ok(main) = instance.exports.get_function("main") {
            debug!("Calling main function");
            match main.call(&mut store, &[]) {
                Ok(_) => {
                    info!("WASM instance started successfully: {}", container_id);
                },
                Err(e) => {
                    error!("Error in WASM execution: {}", e);
                    // Update status to failed
                    if let Some(instance_info) = instances.get_mut(&container_id) {
                        instance_info.status = WasmInstanceStatus::Failed(e.to_string());
                    }
                    return Err(anyhow::anyhow!("WASM execution failed: {}", e));
                }
            }
        } else {
            warn!("No _start or main function found in WASM module");
            // Update status to failed
            if let Some(instance_info) = instances.get_mut(&container_id) {
                instance_info.status = WasmInstanceStatus::Failed("No entry point found".to_string());
            }
            return Err(anyhow::anyhow!("No _start or main function found in WASM module"));
        }
    }
    
    info!("Container {} (ID: {}) is running", container.name, container_id);
    Ok(container_id)
}

/// Stop a running container
pub fn stop_container(container_id: &str) -> Result<()> {
    info!("Stopping container: {}", container_id);
    
    let mut instances = WASM_INSTANCES.lock().unwrap();
    
    if let Some(instance_info) = instances.get_mut(container_id) {
        // Update status to stopped
        instance_info.status = WasmInstanceStatus::Exited(0);
        
        info!("Container stopped: {}", container_id);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Container not found: {}", container_id))
    }
}

/// Get container status
pub fn get_container_status(container_id: &str) -> Result<ContainerStatus> {
    let instances = WASM_INSTANCES.lock().unwrap();
    
    if let Some(instance_info) = instances.get(container_id) {
        let status = match &instance_info.status {
            WasmInstanceStatus::Created => ContainerStatus::Created,
            WasmInstanceStatus::Running => ContainerStatus::Running,
            WasmInstanceStatus::Paused => ContainerStatus::Paused,
            WasmInstanceStatus::Exited(code) => ContainerStatus::Exited(*code),
            WasmInstanceStatus::Failed(msg) => ContainerStatus::Failed(msg.clone()),
        };
        
        Ok(status)
    } else {
        Err(anyhow::anyhow!("Container not found: {}", container_id))
    }
}

/// List all running WASM instances
pub fn list_instances() -> Vec<WasmInstanceInfo> {
    let instances = WASM_INSTANCES.lock().unwrap();
    instances.values().cloned().collect()
}

/// Information about a running WASM instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmInstanceInfo {
    /// Container ID
    pub container_id: String,
    
    /// Container name
    pub container_name: String,
    
    /// Start time
    pub start_time: String,
    
    /// Current status
    pub status: WasmInstanceStatus,
    
    /// Memory usage in bytes
    pub memory_usage: u64,
}

/// WASM instance status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WasmInstanceStatus {
    /// Instance is created
    Created,
    
    /// Instance is running
    Running,
    
    /// Instance is paused
    Paused,
    
    /// Instance has exited
    Exited(i32),
    
    /// Instance has failed
    Failed(String),
}
