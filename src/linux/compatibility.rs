// SentientOS Linux Compatibility Layer
// Provides syscall translation and ELF binary execution

use anyhow::{Result, Context, anyhow};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::process::{Command, Stdio};

use crate::core::constants;
use crate::matrixbox::{self, container::Container};
use crate::zk;

// Global registry for running Linux programs
lazy_static::lazy_static! {
    static ref LINUX_PROCESSES: Arc<Mutex<HashMap<String, LinuxProcess>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

/// Initialize the Linux compatibility subsystem
pub fn init() -> Result<()> {
    info!("Initializing Linux compatibility layer");
    
    // Create necessary directories
    let linux_dir = PathBuf::from(constants::ROOT_DIR).join(".linux");
    fs::create_dir_all(&linux_dir)?;
    
    let bin_dir = linux_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;
    
    let lib_dir = linux_dir.join("lib");
    fs::create_dir_all(&lib_dir)?;
    
    let etc_dir = linux_dir.join("etc");
    fs::create_dir_all(&etc_dir)?;
    
    // Create base configuration files
    let passwd_path = etc_dir.join("passwd");
    if !passwd_path.exists() {
        fs::write(&passwd_path, "root:x:0:0:root:/root:/bin/bash\nsentinent:x:1000:1000:sentinent:/home/sentinent:/bin/bash\n")?;
    }
    
    let group_path = etc_dir.join("group");
    if !group_path.exists() {
        fs::write(&group_path, "root:x:0:\nsentinent:x:1000:\n")?;
    }
    
    // Clear the process registry
    let mut processes = LINUX_PROCESSES.lock().unwrap();
    processes.clear();
    
    info!("Linux compatibility layer initialized successfully");
    Ok(())
}

/// Shutdown the Linux compatibility subsystem
pub fn shutdown() -> Result<()> {
    info!("Shutting down Linux compatibility layer");
    
    // Stop all running processes
    let mut processes = LINUX_PROCESSES.lock().unwrap();
    for (process_id, process) in processes.drain() {
        if let Some(child) = process.process_handle {
            info!("Stopping Linux process: {} (PID: {})", process_id, child.id());
            let _ = child.kill();
        }
    }
    
    info!("Linux compatibility layer shutdown complete");
    Ok(())
}

/// Run an ELF binary in the compatibility layer
pub fn run_elf(path: &Path, args: &[&str]) -> Result<String> {
    info!("Running ELF binary: {:?}", path);
    
    if !path.exists() {
        return Err(anyhow!("ELF binary not found: {:?}", path));
    }
    
    // Verify file is an ELF binary
    if !is_elf_binary(path)? {
        return Err(anyhow!("Not an ELF binary: {:?}", path));
    }
    
    // Generate process ID
    let process_id = generate_process_id();
    
    // Create environment variables
    let mut envs = HashMap::new();
    envs.insert("PATH".to_string(), "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string());
    envs.insert("HOME".to_string(), "/home/sentinent".to_string());
    envs.insert("USER".to_string(), "sentinent".to_string());
    envs.insert("TERM".to_string(), "xterm-256color".to_string());
    
    // Setup process
    let mut command = Command::new(path);
    command
        .args(args)
        .envs(&envs)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    // Start the process
    match command.spawn() {
        Ok(child) => {
            // Register the process
            let process = LinuxProcess {
                id: process_id.clone(),
                path: path.to_path_buf(),
                args: args.iter().map(|s| s.to_string()).collect(),
                start_time: chrono::Utc::now().to_rfc3339(),
                process_handle: Some(child),
                container: None,
            };
            
            let mut processes = LINUX_PROCESSES.lock().unwrap();
            processes.insert(process_id.clone(), process);
            
            info!("Started Linux process: {}", process_id);
            Ok(process_id)
        },
        Err(e) => Err(anyhow!("Failed to start ELF binary: {}", e)),
    }
}

/// Run an ELF binary inside a MatrixBox container
pub fn run_elf_in_container(path: &Path, args: &[&str], container_name: &str) -> Result<String> {
    info!("Running ELF binary in container {}: {:?}", container_name, path);
    
    if !path.exists() {
        return Err(anyhow!("ELF binary not found: {:?}", path));
    }
    
    // Verify file is an ELF binary
    if !is_elf_binary(path)? {
        return Err(anyhow!("Not an ELF binary: {:?}", path));
    }
    
    // Create or get container
    let container = match matrixbox::container::get_container(container_name) {
        Ok(container) => container,
        Err(_) => {
            // Create a new container
            let container_path = PathBuf::from(constants::ROOT_DIR)
                .join(".matrixbox")
                .join("containers")
                .join(container_name);
            
            fs::create_dir_all(&container_path)?;
            
            // Create basic container structure
            let container = Container {
                name: container_name.to_string(),
                version: "1.0.0".to_string(),
                id: None,
                path: Some(container_path.clone()),
                metadata: matrixbox::container::ContainerMetadata {
                    description: format!("Container for ELF binary: {:?}", path),
                    author: "Sentinent OS".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    labels: vec![
                        "linux".to_string(),
                        "elf".to_string(),
                    ],
                    environment: vec![
                        "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
                        "HOME=/home/sentinent".to_string(),
                        "USER=sentinent".to_string(),
                        "TERM=xterm-256color".to_string(),
                    ],
                },
                permissions: matrixbox::container::ContainerPermissions {
                    filesystem: vec![
                        ".".to_string(),
                        "/tmp".to_string(),
                        "/home/sentinent".to_string(),
                    ],
                    network: vec![
                        "localhost:*".to_string(),
                    ],
                    capabilities: vec![
                        "fs.read".to_string(),
                        "fs.write".to_string(),
                        "net.connect".to_string(),
                    ],
                },
            };
            
            // Save container metadata
            matrixbox::container::save_container(&container)?;
            
            container
        }
    };
    
    // Generate process ID
    let process_id = generate_process_id();
    
    // Create environment variables
    let mut envs = HashMap::new();
    envs.insert("PATH".to_string(), "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string());
    envs.insert("HOME".to_string(), "/home/sentinent".to_string());
    envs.insert("USER".to_string(), "sentinent".to_string());
    envs.insert("TERM".to_string(), "xterm-256color".to_string());
    envs.insert("CONTAINER".to_string(), container_name.to_string());
    
    // Setup process
    let mut command = Command::new(path);
    command
        .args(args)
        .envs(&envs)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    // Start the process
    match command.spawn() {
        Ok(child) => {
            // Register the process
            let process = LinuxProcess {
                id: process_id.clone(),
                path: path.to_path_buf(),
                args: args.iter().map(|s| s.to_string()).collect(),
                start_time: chrono::Utc::now().to_rfc3339(),
                process_handle: Some(child),
                container: Some(container),
            };
            
            let mut processes = LINUX_PROCESSES.lock().unwrap();
            processes.insert(process_id.clone(), process);
            
            info!("Started Linux process in container: {} (Container: {})", process_id, container_name);
            Ok(process_id)
        },
        Err(e) => Err(anyhow!("Failed to start ELF binary in container: {}", e)),
    }
}

/// Stop a running Linux process
pub fn stop_process(process_id: &str) -> Result<()> {
    info!("Stopping Linux process: {}", process_id);
    
    let mut processes = LINUX_PROCESSES.lock().unwrap();
    
    if let Some(process) = processes.get_mut(process_id) {
        if let Some(child) = &process.process_handle {
            let pid = child.id();
            match child.kill() {
                Ok(_) => {
                    info!("Stopped Linux process: {} (PID: {})", process_id, pid);
                    process.process_handle = None;
                    Ok(())
                },
                Err(e) => Err(anyhow!("Failed to stop Linux process: {}", e)),
            }
        } else {
            info!("Process already stopped: {}", process_id);
            Ok(())
        }
    } else {
        Err(anyhow!("Process not found: {}", process_id))
    }
}

/// Get status of a Linux process
pub fn get_process_status(process_id: &str) -> Result<ProcessStatus> {
    let processes = LINUX_PROCESSES.lock().unwrap();
    
    if let Some(process) = processes.get(process_id) {
        if let Some(child) = &process.process_handle {
            match child.try_wait() {
                Ok(None) => Ok(ProcessStatus::Running),
                Ok(Some(status)) => {
                    if status.success() {
                        Ok(ProcessStatus::Exited(status.code().unwrap_or(0)))
                    } else {
                        Ok(ProcessStatus::Failed(status.code().unwrap_or(1)))
                    }
                },
                Err(e) => Ok(ProcessStatus::Failed(e.raw_os_error().unwrap_or(1))),
            }
        } else {
            Ok(ProcessStatus::Stopped)
        }
    } else {
        Err(anyhow!("Process not found: {}", process_id))
    }
}

/// List all Linux processes
pub fn list_processes() -> Vec<LinuxProcessInfo> {
    let processes = LINUX_PROCESSES.lock().unwrap();
    
    processes.values()
        .map(|p| LinuxProcessInfo {
            id: p.id.clone(),
            path: p.path.display().to_string(),
            args: p.args.clone(),
            start_time: p.start_time.clone(),
            container_name: p.container.as_ref().map(|c| c.name.clone()),
            status: match &p.process_handle {
                Some(child) => match child.try_wait() {
                    Ok(None) => ProcessStatus::Running,
                    Ok(Some(status)) => {
                        if status.success() {
                            ProcessStatus::Exited(status.code().unwrap_or(0))
                        } else {
                            ProcessStatus::Failed(status.code().unwrap_or(1))
                        }
                    },
                    Err(_) => ProcessStatus::Failed(1),
                },
                None => ProcessStatus::Stopped,
            },
        })
        .collect()
}

/// Check if a file is an ELF binary
fn is_elf_binary(path: &Path) -> Result<bool> {
    // Open the file
    let mut file = fs::File::open(path)?;
    
    // Read the magic number (4 bytes)
    let mut magic = [0u8; 4];
    use std::io::Read;
    file.read_exact(&mut magic)?;
    
    // Check for ELF magic number: 0x7F, 'E', 'L', 'F'
    Ok(magic[0] == 0x7F && magic[1] == b'E' && magic[2] == b'L' && magic[3] == b'F')
}

/// Generate a unique process ID
fn generate_process_id() -> String {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    
    let timestamp = chrono::Utc::now().timestamp();
    let random: u32 = rng.gen();
    
    format!("proc-{}-{:08x}", timestamp, random)
}

/// Linux process information
#[derive(Debug, Clone)]
struct LinuxProcess {
    /// Process ID
    id: String,
    
    /// Path to the executable
    path: PathBuf,
    
    /// Command line arguments
    args: Vec<String>,
    
    /// Start time
    start_time: String,
    
    /// Process handle
    process_handle: Option<std::process::Child>,
    
    /// Container (if running in a container)
    container: Option<Container>,
}

/// Linux process information for API
#[derive(Debug, Clone)]
pub struct LinuxProcessInfo {
    /// Process ID
    pub id: String,
    
    /// Path to the executable
    pub path: String,
    
    /// Command line arguments
    pub args: Vec<String>,
    
    /// Start time
    pub start_time: String,
    
    /// Container name (if running in a container)
    pub container_name: Option<String>,
    
    /// Process status
    pub status: ProcessStatus,
}

/// Process status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Process is running
    Running,
    
    /// Process has been stopped
    Stopped,
    
    /// Process has exited with a status code
    Exited(i32),
    
    /// Process has failed with an error code
    Failed(i32),
}
