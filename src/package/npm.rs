// SentientOS Package Manager - NPM Package Handler
// Handles Node.js packages using npm

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::process::Command;
use std::path::PathBuf;
use crate::core::constants;

/// Install an npm package
pub fn install_package(name: &str, version: Option<&str>) -> Result<()> {
    info!("Installing npm package: {}", name);
    
    // Check if npm is installed
    let npm_check = Command::new("which")
        .arg("npm")
        .output()?;
        
    if !npm_check.status.success() {
        return Err(anyhow::anyhow!("npm not found, please install Node.js"));
    }
    
    // Create package directory
    let npm_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("npm");
    std::fs::create_dir_all(&npm_dir)?;
    
    // Run npm install
    let mut cmd = Command::new("npm");
    cmd.current_dir(&npm_dir);
    cmd.arg("install");
    
    // Add global flag for system-wide packages
    cmd.arg("--global");
    
    if let Some(ver) = version {
        cmd.arg(format!("{}@{}", name, ver));
    } else {
        cmd.arg(name);
    }
    
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to install npm package: {}\n{}", name, stderr));
    }
    
    info!("npm package {} installed successfully", name);
    Ok(())
}

/// Remove an npm package
pub fn remove_package(name: &str) -> Result<()> {
    info!("Removing npm package: {}", name);
    
    // Check if npm is installed
    let npm_check = Command::new("which")
        .arg("npm")
        .output()?;
        
    if !npm_check.status.success() {
        return Err(anyhow::anyhow!("npm not found, please install Node.js"));
    }
    
    // Run npm uninstall
    let mut cmd = Command::new("npm");
    cmd.args(["uninstall", "--global", name]);
    
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to remove npm package: {}\n{}", name, stderr));
    }
    
    info!("npm package {} removed successfully", name);
    Ok(())
}

/// Run an npm package with arguments
pub fn run_package(name: &str, args: &[&str]) -> Result<()> {
    info!("Running npm package: {}", name);
    
    // First try npx
    let mut cmd = Command::new("npx");
    cmd.arg(name);
    cmd.args(args);
    
    // Run in a subshell to handle shebang scripts properly
    let mut child = cmd.spawn()?;
    let status = child.wait()?;
    
    if !status.success() {
        warn!("npx command failed, trying node_modules/.bin directory");
        
        // Try looking in the standard binary paths
        let home = std::env::var("HOME").unwrap_or_default();
        let bin_paths = [
            format!("{}/node_modules/.bin/{}", home, name),
            format!("/usr/local/bin/{}", name),
            format!("/usr/bin/{}", name),
        ];
        
        for path in bin_paths {
            if std::path::Path::new(&path).exists() {
                let mut cmd = Command::new(&path);
                cmd.args(args);
                
                let mut child = cmd.spawn()?;
                child.wait()?;
                return Ok(());
            }
        }
        
        return Err(anyhow::anyhow!("Failed to run npm package: {}", name));
    }
    
    Ok(())
}

/// Search for npm packages
pub fn search_packages(query: &str) -> Result<Vec<String>> {
    info!("Searching for npm packages matching: {}", query);
    
    // Check if npm is installed
    let npm_check = Command::new("which")
        .arg("npm")
        .output()?;
        
    if !npm_check.status.success() {
        return Err(anyhow::anyhow!("npm not found, please install Node.js"));
    }
    
    let mut results = Vec::new();
    
    // Run npm search
    let cmd = Command::new("npm")
        .args(["search", query, "--no-description", "--parseable"])
        .output()?;
        
    if cmd.status.success() {
        let output = String::from_utf8_lossy(&cmd.stdout);
        for line in output.lines() {
            if !line.is_empty() {
                let parts: Vec<&str> = line.split("\t").collect();
                if !parts.is_empty() {
                    // Format: name<tab>description<tab>version<tab>date
                    let pkg_name = parts[0];
                    let description = if parts.len() > 1 { parts[1] } else { "" };
                    
                    results.push(format!("{} (npm) - {}", pkg_name, description));
                    
                    // Limit results to avoid overwhelming output
                    if results.len() >= 10 {
                        break;
                    }
                }
            }
        }
    } else {
        warn!("npm search failed: {}", String::from_utf8_lossy(&cmd.stderr));
    }
    
    Ok(results)
}
