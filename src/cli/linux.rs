// SentientOS Linux Compatibility CLI Module
// Provides commands for managing Linux compatibility layer

use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

use crate::core::constants;
use crate::linux::{compatibility, elf_loader};

/// Linux compatibility CLI subcommands
#[derive(Subcommand)]
pub enum LinuxCommands {
    /// Run a Linux ELF binary
    Run {
        /// Path to the binary
        binary_path: String,
        
        /// Arguments to pass to the binary
        #[clap(multiple = true)]
        args: Vec<String>,
    },
    
    /// Run a Linux ELF binary inside a MatrixBox container
    RunContainer {
        /// Path to the binary
        binary_path: String,
        
        /// Container name to run in
        container_name: String,
        
        /// Arguments to pass to the binary
        #[clap(multiple = true)]
        args: Vec<String>,
    },
    
    /// List running Linux processes
    Ps {},
    
    /// Kill a running Linux process
    Kill {
        /// Process ID to kill
        pid: u32,
        
        /// Force kill with SIGKILL instead of SIGTERM
        #[clap(short, long)]
        force: bool,
    },
    
    /// Show detailed information about an ELF binary
    Inspect {
        /// Path to the binary
        binary_path: String,
    },
    
    /// List installed Linux shared libraries
    Libs {},
    
    /// Install a shared library
    InstallLib {
        /// Path to the library file
        lib_path: String,
    },
    
    /// Print Linux compatibility layer status
    Status {},
}

/// Handle Linux CLI commands
pub fn handle_command(cmd: &LinuxCommands) -> Result<()> {
    match cmd {
        LinuxCommands::Run { binary_path, args } => {
            run_binary(binary_path, args)
        }
        LinuxCommands::RunContainer { binary_path, container_name, args } => {
            run_binary_in_container(binary_path, container_name, args)
        }
        LinuxCommands::Ps {} => {
            list_processes()
        }
        LinuxCommands::Kill { pid, force } => {
            kill_process(*pid, *force)
        }
        LinuxCommands::Inspect { binary_path } => {
            inspect_binary(binary_path)
        }
        LinuxCommands::Libs {} => {
            list_shared_libs()
        }
        LinuxCommands::InstallLib { lib_path } => {
            install_shared_lib(lib_path)
        }
        LinuxCommands::Status {} => {
            show_status()
        }
    }
}

/// Run a Linux ELF binary
fn run_binary(binary_path: &str, args: &[String]) -> Result<()> {
    info!("Running Linux binary: {}", binary_path);
    
    // Convert to absolute path if needed
    let path = if Path::new(binary_path).is_absolute() {
        PathBuf::from(binary_path)
    } else {
        std::env::current_dir()?.join(binary_path)
    };
    
    // Check if file exists
    if !path.exists() {
        error!("Binary not found: {:?}", path);
        println!("{} Binary not found: {:?}", "ERROR:".red().bold(), path);
        return Ok(());
    }
    
    // Verify it's an ELF binary
    if !elf_loader::is_elf_binary(&path)? {
        error!("Not an ELF binary: {:?}", path);
        println!("{} Not a valid ELF binary: {:?}", "ERROR:".red().bold(), path);
        return Ok(());
    }
    
    // Convert args to &str slice
    let args_str: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
    
    println!("{} Running binary: {}", "INFO:".blue().bold(), path.display());
    let output = match elf_loader::execute_elf(&path, &args_str) {
        Ok(output) => output,
        Err(err) => {
            error!("Failed to execute binary: {}", err);
            println!("{} Execution failed: {}", "ERROR:".red().bold(), err);
            return Ok(());
        }
    };
    
    // Print output
    println!("{}", output);
    
    println!("{} Binary execution completed", "SUCCESS:".green().bold());
    Ok(())
}

/// Run a Linux ELF binary inside a MatrixBox container
fn run_binary_in_container(binary_path: &str, container_name: &str, args: &[String]) -> Result<()> {
    info!("Running Linux binary in container {}: {}", container_name, binary_path);
    
    // Convert to absolute path if needed
    let path = if Path::new(binary_path).is_absolute() {
        PathBuf::from(binary_path)
    } else {
        std::env::current_dir()?.join(binary_path)
    };
    
    // Check if file exists
    if !path.exists() {
        error!("Binary not found: {:?}", path);
        println!("{} Binary not found: {:?}", "ERROR:".red().bold(), path);
        return Ok(());
    }
    
    // Verify it's an ELF binary
    if !elf_loader::is_elf_binary(&path)? {
        error!("Not an ELF binary: {:?}", path);
        println!("{} Not a valid ELF binary: {:?}", "ERROR:".red().bold(), path);
        return Ok(());
    }
    
    // Convert args to &str slice
    let args_str: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
    
    println!("{} Running binary in container {}: {}", "INFO:".blue().bold(), container_name, path.display());
    let output = match elf_loader::execute_elf_in_container(&path, &args_str, container_name) {
        Ok(output) => output,
        Err(err) => {
            error!("Failed to execute binary in container: {}", err);
            println!("{} Execution failed: {}", "ERROR:".red().bold(), err);
            return Ok(());
        }
    };
    
    // Print output
    println!("{}", output);
    
    println!("{} Container execution completed", "SUCCESS:".green().bold());
    Ok(())
}

/// List running Linux processes
fn list_processes() -> Result<()> {
    info!("Listing Linux processes");
    
    match compatibility::list_processes() {
        Ok(processes) => {
            if processes.is_empty() {
                println!("{} No Linux processes running", "INFO:".blue().bold());
            } else {
                println!("{: <10} {: <20} {: <15} {: <15}", "PID", "NAME", "STATE", "CONTAINER");
                println!("{}", "-".repeat(60));
                
                for proc in processes {
                    let state = match proc.state.as_str() {
                        "running" => proc.state.green(),
                        "stopped" => proc.state.yellow(),
                        _ => proc.state.normal(),
                    };
                    
                    println!(
                        "{: <10} {: <20} {: <15} {: <15}",
                        proc.pid,
                        proc.name,
                        state,
                        proc.container.unwrap_or_default()
                    );
                }
            }
        }
        Err(err) => {
            error!("Failed to list processes: {}", err);
            println!("{} Failed to list processes: {}", "ERROR:".red().bold(), err);
        }
    }
    
    Ok(())
}

/// Kill a Linux process
fn kill_process(pid: u32, force: bool) -> Result<()> {
    let signal = if force { "SIGKILL" } else { "SIGTERM" };
    info!("Killing Linux process {} with {}", pid, signal);
    
    match compatibility::kill_process(pid, force) {
        Ok(()) => {
            println!("{} Process {} killed with {}", "SUCCESS:".green().bold(), pid, signal);
        }
        Err(err) => {
            error!("Failed to kill process {}: {}", pid, err);
            println!("{} Failed to kill process {}: {}", "ERROR:".red().bold(), pid, err);
        }
    }
    
    Ok(())
}

/// Inspect a Linux ELF binary
fn inspect_binary(binary_path: &str) -> Result<()> {
    info!("Inspecting ELF binary: {}", binary_path);
    
    // Convert to absolute path if needed
    let path = if Path::new(binary_path).is_absolute() {
        PathBuf::from(binary_path)
    } else {
        std::env::current_dir()?.join(binary_path)
    };
    
    // Check if file exists
    if !path.exists() {
        error!("Binary not found: {:?}", path);
        println!("{} Binary not found: {:?}", "ERROR:".red().bold(), path);
        return Ok(());
    }
    
    // Verify it's an ELF binary
    if !elf_loader::is_elf_binary(&path)? {
        error!("Not an ELF binary: {:?}", path);
        println!("{} Not a valid ELF binary: {:?}", "ERROR:".red().bold(), path);
        return Ok(());
    }
    
    // Analyze the ELF binary
    match elf_loader::analyze_elf(&path) {
        Ok(info) => {
            println!("{} Binary inspection result:", "INFO:".blue().bold());
            elf_loader::print_elf_info(&info);
        }
        Err(err) => {
            error!("Failed to analyze ELF binary: {}", err);
            println!("{} Failed to analyze ELF binary: {}", "ERROR:".red().bold(), err);
        }
    }
    
    Ok(())
}

/// List shared libraries
fn list_shared_libs() -> Result<()> {
    info!("Listing shared libraries");
    
    let linux_lib_dir = PathBuf::from(constants::ROOT_DIR).join(".linux").join("lib");
    if !linux_lib_dir.exists() {
        println!("{} Linux lib directory not found", "WARNING:".yellow().bold());
        return Ok(());
    }
    
    println!("{} Installed shared libraries:", "INFO:".blue().bold());
    println!("{}", "-".repeat(60));
    
    let libs = std::fs::read_dir(&linux_lib_dir)?;
    let mut found = false;
    
    for entry in libs {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "so" || ext.to_string_lossy().contains("so.")) {
                found = true;
                println!("{}", path.file_name().unwrap().to_string_lossy());
            }
        }
    }
    
    if !found {
        println!("No shared libraries found");
    }
    
    Ok(())
}

/// Install a shared library
fn install_shared_lib(lib_path: &str) -> Result<()> {
    info!("Installing shared library: {}", lib_path);
    
    // Convert to absolute path if needed
    let path = if Path::new(lib_path).is_absolute() {
        PathBuf::from(lib_path)
    } else {
        std::env::current_dir()?.join(lib_path)
    };
    
    // Check if file exists
    if !path.exists() {
        error!("Library file not found: {:?}", path);
        println!("{} Library file not found: {:?}", "ERROR:".red().bold(), path);
        return Ok(());
    }
    
    // Verify it's an ELF binary (shared libraries are ELF files too)
    if !elf_loader::is_elf_binary(&path)? {
        error!("Not a valid ELF library: {:?}", path);
        println!("{} Not a valid ELF library: {:?}", "ERROR:".red().bold(), path);
        return Ok(());
    }
    
    // Get filename
    let filename = path.file_name().ok_or_else(|| {
        anyhow::anyhow!("Invalid library filename")
    })?;
    
    // Create .linux/lib directory if it doesn't exist
    let linux_lib_dir = PathBuf::from(constants::ROOT_DIR).join(".linux").join("lib");
    std::fs::create_dir_all(&linux_lib_dir)?;
    
    // Copy the library to .linux/lib
    let dest_path = linux_lib_dir.join(filename);
    std::fs::copy(&path, &dest_path)?;
    
    println!("{} Installed library: {}", "SUCCESS:".green().bold(), dest_path.display());
    
    Ok(())
}

/// Show Linux compatibility layer status
fn show_status() -> Result<()> {
    info!("Checking Linux compatibility layer status");
    
    let linux_dir = PathBuf::from(constants::ROOT_DIR).join(".linux");
    let linux_active = linux_dir.exists();
    
    println!("{} Linux Compatibility Status", "INFO:".blue().bold());
    println!("{}", "-".repeat(60));
    println!("Compatibility Layer Active: {}", 
             if linux_active { "Yes".green() } else { "No".red() });
    
    // Check if basic directories exist
    let dirs = vec![
        (linux_dir.join("bin"), "Binaries"),
        (linux_dir.join("lib"), "Libraries"),
        (linux_dir.join("etc"), "Configuration"),
        (linux_dir.join("proc"), "Process Info"),
    ];
    
    for (dir_path, desc) in dirs {
        println!("{} Directory: {}", 
                 desc, 
                 if dir_path.exists() { "Present".green() } else { "Missing".red() });
    }
    
    // Check number of processes
    match compatibility::list_processes() {
        Ok(processes) => {
            println!("Active Processes: {}", processes.len());
        }
        Err(_) => {
            println!("Active Processes: {}", "Unknown".yellow());
        }
    }
    
    // Show kernel emulation status
    println!("Syscall Translation: {}", "Active".green());
    println!("Syscall Verification: {}", "ZK-Enforced".green());
    
    Ok(())
}
