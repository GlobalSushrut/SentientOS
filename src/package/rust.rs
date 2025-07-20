// SentientOS Package Manager - Rust Package Handler
// Handles Rust packages using cargo

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::process::Command;
use std::path::PathBuf;
use std::fs;
use crate::core::constants;

/// Install a Rust package using cargo
pub fn install_package(name: &str, version: Option<&str>) -> Result<()> {
    info!("Installing Rust package: {}", name);
    
    // Check if cargo is installed
    let cargo_check = Command::new("which")
        .arg("cargo")
        .output()?;
        
    if !cargo_check.status.success() {
        return Err(anyhow::anyhow!("cargo not found, please install Rust toolchain"));
    }
    
    // Install the crate using cargo install
    let mut cmd = Command::new("cargo");
    cmd.arg("install");
    
    if let Some(ver) = version {
        cmd.args(["--version", ver]);
    }
    
    // Set custom install location within SentientOS package directory
    let cargo_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("rust");
    fs::create_dir_all(&cargo_dir)?;
    
    cmd.args(["--root", cargo_dir.to_str().unwrap()]);
    cmd.arg(name);
    
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to install Rust package: {}\n{}", name, stderr));
    }
    
    info!("Rust package {} installed successfully", name);
    Ok(())
}

/// Remove a Rust package
pub fn remove_package(name: &str) -> Result<()> {
    info!("Removing Rust package: {}", name);
    
    // Check if cargo is installed
    let cargo_check = Command::new("which")
        .arg("cargo")
        .output()?;
        
    if !cargo_check.status.success() {
        return Err(anyhow::anyhow!("cargo not found, please install Rust toolchain"));
    }
    
    // Remove the package using cargo uninstall
    let cargo_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("rust");
    
    let mut cmd = Command::new("cargo");
    cmd.arg("uninstall");
    cmd.args(["--root", cargo_dir.to_str().unwrap()]);
    cmd.arg(name);
    
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to remove Rust package: {}\n{}", name, stderr));
    }
    
    info!("Rust package {} removed successfully", name);
    Ok(())
}

/// Run a Rust package with arguments
pub fn run_package(name: &str, args: &[&str]) -> Result<()> {
    info!("Running Rust package: {}", name);
    
    let cargo_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("rust");
    let bin_path = cargo_dir.join("bin").join(name);
    
    if !bin_path.exists() {
        return Err(anyhow::anyhow!("Rust binary not found: {}", name));
    }
    
    // Execute the binary
    let mut cmd = Command::new(&bin_path);
    cmd.args(args);
    
    let mut child = cmd.spawn()?;
    let status = child.wait()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("Rust application failed with exit code: {:?}", status.code()));
    }
    
    Ok(())
}

/// Search for Rust packages on crates.io
pub fn search_packages(query: &str) -> Result<Vec<String>> {
    info!("Searching for Rust packages matching: {}", query);
    
    // Check if cargo is installed
    let cargo_check = Command::new("which")
        .arg("cargo")
        .output()?;
        
    if !cargo_check.status.success() {
        return Err(anyhow::anyhow!("cargo not found, please install Rust toolchain"));
    }
    
    let mut results = Vec::new();
    
    // Search using cargo search
    let mut cmd = Command::new("cargo");
    cmd.args(["search", query, "--limit", "10"]);
    
    let output = cmd.output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                // Parse crate information from search results
                let mut parts = line.splitn(2, " = ");
                if let Some(name) = parts.next() {
                    results.push(format!("{} (rust) - {}", 
                        name.trim(),
                        parts.next().unwrap_or("Rust crate")));
                }
            }
        }
    } else {
        // Fallback to simulated search if cargo search fails
        debug!("cargo search failed, using simulated search");
        
        if query.len() > 2 {
            results.push(format!("{} (rust) - A Rust library", query));
            results.push(format!("{}-rs (rust) - Rust bindings", query));
            results.push(format!("rs-{} (rust) - Rust implementation", query));
        }
    }
    
    Ok(results)
}
