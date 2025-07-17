use anyhow::{Result, Context};
use tracing::{info, warn, error};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmer::{Store, Module, Instance, Memory, MemoryType};
use wasmer::AsStoreRef;
use wasmer_wasi::{WasiEnv, WasiState};
use std::path::PathBuf;
use std::fs;

use super::container::{Container, ContainerId, ContainerStatus};
use super::registry;
use crate::core::constants;

// Map of container ID to running instance
lazy_static::lazy_static! {
    static ref RUNNING_CONTAINERS: Arc<Mutex<HashMap<ContainerId, RunningContainer>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

/// Running container instance
struct RunningContainer {
    /// Container ID
    id: ContainerId,
    
    /// Wasmer store
    store: Store,
    
    /// Wasmer module
    module: Module,
    
    /// Wasmer instance
    instance: Instance,
    
    /// WASI environment
    wasi_env: WasiEnv,
    
    /// Memory snapshot for ZK verification
    memory_snapshots: Vec<Vec<u8>>,
}

/// Initialize the MatrixBox runtime
pub fn init() -> Result<()> {
    info!("Initializing MatrixBox runtime");
    
    // Create runtime directories
    let runtime_dir = PathBuf::from(constants::ROOT_DIR)
        .join(constants::CONTAINER_DIR)
        .join("runtime");
    
    fs::create_dir_all(&runtime_dir)
        .context("Failed to create runtime directory")?;
    
    info!("MatrixBox runtime initialized successfully");
    Ok(())
}

/// Shutdown the MatrixBox runtime
pub fn shutdown() -> Result<()> {
    info!("Shutting down MatrixBox runtime");
    
    // Stop all running containers
    let mut running_containers = RUNNING_CONTAINERS.lock().unwrap();
    let ids: Vec<ContainerId> = running_containers.keys().cloned().collect();
    
    for id in ids {
        match stop_container_internal(&id, &mut running_containers) {
            Ok(_) => info!("Stopped container: {}", id),
            Err(e) => warn!("Failed to stop container {}: {}", id, e),
        }
    }
    
    running_containers.clear();
    
    info!("MatrixBox runtime shutdown complete");
    Ok(())
}

/// Start a container
pub fn start_container(id: &ContainerId) -> Result<()> {
    info!("Starting container: {}", id);
    
    // Get the container from registry
    let container = registry::get_container(id)?;
    
    // Check if container is already running
    {
        let running_containers = RUNNING_CONTAINERS.lock().unwrap();
        if running_containers.contains_key(id) {
            anyhow::bail!("Container is already running: {}", id);
        }
    }
    
    // Get the container path
    let container_path = container.path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Container has no path"))?;
    
    // Load the WASM module
    let wasm_path = container_path.join("main.wasm");
    let wasm_bytes = fs::read(&wasm_path)
        .with_context(|| format!("Failed to read WASM file: {:?}", wasm_path))?;
    
    // Create WASI environment
    let mut wasi_state = WasiState::new("sentientos-matrixbox");
    
    // Set environment variables
    for env_var in &container.metadata.environment {
        if let Some(pos) = env_var.find('=') {
            let key = &env_var[0..pos];
            let value = &env_var[pos+1..];
            wasi_state = wasi_state.env(key, value);
        }
    }
    
    // Add standard directories
    wasi_state = wasi_state
        .preopen_dir(container_path, "/")?
        .preopen_dir(PathBuf::from(constants::ROOT_DIR).join(".runtime"), "/runtime")?;
    
    // Add container-specific permissions
    for path in &container.permissions.filesystem {
        let fs_path = PathBuf::from(constants::ROOT_DIR).join(path);
        if fs_path.exists() {
            let mount_point = format!("/{}", path);
            wasi_state = wasi_state.preopen_dir(fs_path, mount_point)?;
        }
    }
    
    // Create the WASI environment
    let wasi_env = wasi_state.finalize()?;
    
    // Create the Wasmer store and compile module
    let mut store = Store::default();
    let module = Module::new(&store, wasm_bytes)
        .context("Failed to compile WASM module")?;
    
    // Create import object for WASI
    let import_object = wasi_env.import_object(&mut store, &module)?;
    
    // Instantiate module
    let instance = Instance::new(&mut store, &module, &import_object)
        .context("Failed to instantiate WASM module")?;
    
    // Create running container
    let running_container = RunningContainer {
        id: id.clone(),
        store,
        module,
        instance,
        wasi_env,
        memory_snapshots: Vec::new(),
    };
    
    // Add to running containers
    {
        let mut running_containers = RUNNING_CONTAINERS.lock().unwrap();
        running_containers.insert(id.clone(), running_container);
    }
    
    // Update container status
    registry::update_container_status(id, ContainerStatus::Running)?;
    
    info!("Container started: {}", id);
    Ok(())
}

/// Stop a container
pub fn stop_container(id: &ContainerId) -> Result<()> {
    info!("Stopping container: {}", id);
    
    let mut running_containers = RUNNING_CONTAINERS.lock().unwrap();
    stop_container_internal(id, &mut running_containers)
}

/// Internal function to stop a container
fn stop_container_internal(
    id: &ContainerId, 
    running_containers: &mut HashMap<ContainerId, RunningContainer>
) -> Result<()> {
    // Remove container from running containers
    if running_containers.remove(id).is_some() {
        // Update container status
        registry::update_container_status(id, ContainerStatus::Exited(0))?;
        
        info!("Container stopped: {}", id);
        Ok(())
    } else {
        anyhow::bail!("Container is not running: {}", id);
    }
}

/// Check if a container is running
pub fn is_container_running(id: &ContainerId) -> Result<bool> {
    let running_containers = RUNNING_CONTAINERS.lock().unwrap();
    Ok(running_containers.contains_key(id))
}

/// Take a memory snapshot for ZK verification
pub fn take_memory_snapshot(id: &ContainerId) -> Result<()> {
    info!("Taking memory snapshot for container: {}", id);
    
    let mut running_containers = RUNNING_CONTAINERS.lock().unwrap();
    
    if let Some(container) = running_containers.get_mut(id) {
        // Get the memory from the instance
        let memory = container.instance
            .exports
            .get_memory("memory")
            .map_err(|_| anyhow::anyhow!("Memory not exported by WASM module"))?;
        
        // Copy the memory data
        let memory_view = memory.view(&container.store.as_store_ref());
        let memory_data = memory_view.data().to_vec();
        
        // Add to snapshots
        container.memory_snapshots.push(memory_data);
        
        info!("Memory snapshot taken for container: {}", id);
        Ok(())
    } else {
        anyhow::bail!("Container is not running: {}", id);
    }
}

/// Verify memory with ZK proofs
pub fn verify_memory_zk(id: &ContainerId) -> Result<bool> {
    info!("Verifying memory with ZK proofs for container: {}", id);
    
    let running_containers = RUNNING_CONTAINERS.lock().unwrap();
    
    if let Some(container) = running_containers.get(id) {
        if container.memory_snapshots.is_empty() {
            warn!("No memory snapshots available for container: {}", id);
            return Ok(false);
        }
        
        // Get the latest memory snapshot
        let snapshot = &container.memory_snapshots[container.memory_snapshots.len() - 1];
        
        // Generate ZK proof for the memory
        let proof = crate::zk::generate_proof(snapshot, "memory_verify")?;
        
        // Verify the proof
        let result = crate::zk::verify_proof(snapshot, &proof, "memory_verify")?;
        
        if result {
            info!("ZK memory verification passed for container: {}", id);
        } else {
            warn!("ZK memory verification failed for container: {}", id);
        }
        
        Ok(result)
    } else {
        anyhow::bail!("Container is not running: {}", id);
    }
}

/// Execute a function in the container
pub fn execute_function(id: &ContainerId, function_name: &str, args: &[wasmer::Value]) -> Result<Vec<wasmer::Value>> {
    info!("Executing function '{}' in container: {}", function_name, id);
    
    let mut running_containers = RUNNING_CONTAINERS.lock().unwrap();
    
    if let Some(container) = running_containers.get_mut(id) {
        // Get the function from the instance
        let function = container.instance
            .exports
            .get_function(function_name)
            .with_context(|| format!("Function '{}' not found in container", function_name))?;
        
        // Take pre-execution memory snapshot for verification
        take_memory_snapshot_internal(container)?;
        
        // Call the function
        let results = function.call(&mut container.store.as_store_ref(), args)
            .with_context(|| format!("Failed to execute function '{}'", function_name))?;
        
        // Take post-execution memory snapshot for verification
        take_memory_snapshot_internal(container)?;
        
        info!("Function '{}' executed successfully in container: {}", function_name, id);
        Ok(results)
    } else {
        anyhow::bail!("Container is not running: {}", id);
    }
}

/// Internal function to take memory snapshot
fn take_memory_snapshot_internal(container: &mut RunningContainer) -> Result<()> {
    // Get the memory from the instance
    let memory = container.instance
        .exports
        .get_memory("memory")
        .map_err(|_| anyhow::anyhow!("Memory not exported by WASM module"))?;
    
    // Copy the memory data
    let memory_view = memory.view(&container.store.as_store_ref());
    let memory_data = memory_view.data().to_vec();
    
    // Add to snapshots
    container.memory_snapshots.push(memory_data);
    
    Ok(())
}
