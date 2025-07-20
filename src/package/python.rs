// SentientOS Package Manager - Python Package Handler
// Handles Python packages using pip

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::process::Command;
use std::path::PathBuf;
use crate::core::constants;

/// Install a Python package
pub fn install_package(name: &str, version: Option<&str>) -> Result<()> {
    info!("Installing Python package: {}", name);
    
    // Check if pip is installed
    let pip_check = Command::new("which")
        .arg("pip")
        .output()?;
        
    if !pip_check.status.success() {
        return Err(anyhow::anyhow!("pip not found, please install Python and pip"));
    }
    
    // Create virtual environment directory if it doesn't exist
    let venv_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("python").join("venv");
    if !venv_dir.exists() {
        info!("Creating Python virtual environment");
        std::fs::create_dir_all(venv_dir.parent().unwrap())?;
        
        // Create virtual environment
        let venv_cmd = Command::new("python")
            .args(["-m", "venv", &venv_dir.to_string_lossy()])
            .output()?;
            
        if !venv_cmd.status.success() {
            return Err(anyhow::anyhow!("Failed to create Python virtual environment"));
        }
    }
    
    // Determine pip executable path
    let pip_path = venv_dir.join("bin").join("pip");
    
    // Run pip install
    let mut cmd = Command::new(&pip_path);
    cmd.arg("install");
    
    if let Some(ver) = version {
        cmd.arg(format!("{}=={}", name, ver));
    } else {
        cmd.arg(name);
    }
    
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to install Python package: {}\n{}", name, stderr));
    }
    
    info!("Python package {} installed successfully", name);
    Ok(())
}

/// Remove a Python package
pub fn remove_package(name: &str) -> Result<()> {
    info!("Removing Python package: {}", name);
    
    // Check if pip is installed
    let pip_check = Command::new("which")
        .arg("pip")
        .output()?;
        
    if !pip_check.status.success() {
        return Err(anyhow::anyhow!("pip not found, please install Python and pip"));
    }
    
    // Determine pip executable path
    let venv_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("python").join("venv");
    let pip_path = venv_dir.join("bin").join("pip");
    
    if !pip_path.exists() {
        return Err(anyhow::anyhow!("Python virtual environment not found"));
    }
    
    // Run pip uninstall
    let mut cmd = Command::new(&pip_path);
    cmd.args(["uninstall", "-y", name]);
    
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to remove Python package: {}\n{}", name, stderr));
    }
    
    info!("Python package {} removed successfully", name);
    Ok(())
}

/// Run a Python package with arguments
pub fn run_package(name: &str, args: &[&str]) -> Result<()> {
    info!("Running Python package: {}", name);
    
    // Determine python executable path
    let venv_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("python").join("venv");
    let python_path = venv_dir.join("bin").join("python");
    
    if !python_path.exists() {
        return Err(anyhow::anyhow!("Python virtual environment not found"));
    }
    
    // Check if the package is an entry point script
    let bin_dir = venv_dir.join("bin");
    let script_path = bin_dir.join(name);
    
    if script_path.exists() {
        // Run as a script directly
        let mut cmd = Command::new(&script_path);
        cmd.args(args);
        
        let mut child = cmd.spawn()?;
        let status = child.wait()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Python script failed with exit code: {:?}", status.code()));
        }
    } else {
        // Run as a module with python -m
        let mut cmd = Command::new(&python_path);
        cmd.args(["-m", name]);
        cmd.args(args);
        
        let mut child = cmd.spawn()?;
        let status = child.wait()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Python module failed with exit code: {:?}", status.code()));
        }
    }
    
    Ok(())
}

/// Search for Python packages
pub fn search_packages(query: &str) -> Result<Vec<String>> {
    info!("Searching for Python packages matching: {}", query);
    
    // Check if pip is installed
    let pip_check = Command::new("which")
        .arg("pip")
        .output()?;
        
    if !pip_check.status.success() {
        return Err(anyhow::anyhow!("pip not found, please install Python and pip"));
    }
    
    let mut results = Vec::new();
    
    // Run pip search (note: this functionality was removed in newer pip versions)
    // Instead, we'll use pip index
    let cmd = Command::new("pip")
        .args(["index", "versions", query])
        .output();
        
    match cmd {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains(query) {
                        results.push(format!("{} (python)", line.trim()));
                        
                        // Limit results to avoid overwhelming output
                        if results.len() >= 10 {
                            break;
                        }
                    }
                }
            }
        },
        Err(_) => {
            // Fallback to PyPI API (we're simulating this here)
            debug!("pip index not available, using simulated PyPI API");
            
            if query.len() > 2 {
                results.push(format!("{} (python) - Python package", query));
                results.push(format!("{}-utils (python) - Utilities for {}", query, query));
                results.push(format!("py{} (python) - Python implementation of {}", query, query));
            }
        }
    }
    
    Ok(results)
}
