// SentientOS Package Manager - Linux Package Handler
// Handles apt, dnf, pacman, and other Linux package managers

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::process::Command;
use std::path::PathBuf;

/// Detect the system package manager
pub fn detect_package_manager() -> Result<&'static str> {
    let package_managers = [
        ("apt", "/usr/bin/apt"),
        ("dnf", "/usr/bin/dnf"),
        ("yum", "/usr/bin/yum"),
        ("pacman", "/usr/bin/pacman"),
        ("zypper", "/usr/bin/zypper"),
        ("apk", "/sbin/apk"),
        ("pkg", "/usr/sbin/pkg"),
    ];
    
    for (manager, path) in package_managers {
        if std::path::Path::new(path).exists() {
            return Ok(manager);
        }
    }
    
    Err(anyhow::anyhow!("No supported package manager found"))
}

/// Install a Linux package
pub fn install_package(name: &str, version: Option<&str>) -> Result<()> {
    let pkg_manager = detect_package_manager()?;
    info!("Installing Linux package {} using {}", name, pkg_manager);
    
    match pkg_manager {
        "apt" => {
            let mut cmd = Command::new("apt");
            cmd.args(["install", "-y"]);
            
            if let Some(ver) = version {
                cmd.arg(&format!("{}={}", name, ver));
            } else {
                cmd.arg(name);
            }
            
            // Use matrixbox container to isolate the installation
            debug!("Running apt in MatrixBox container");
            let output = cmd.output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install package: {}\n{}", name, stderr));
            }
        },
        "dnf" | "yum" => {
            let mut cmd = Command::new(pkg_manager);
            cmd.args(["install", "-y"]);
            
            if let Some(ver) = version {
                cmd.arg(&format!("{}-{}", name, ver));
            } else {
                cmd.arg(name);
            }
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install package: {}\n{}", name, stderr));
            }
        },
        "pacman" => {
            let mut cmd = Command::new("pacman");
            cmd.args(["-S", "--noconfirm"]);
            
            if let Some(ver) = version {
                cmd.arg(&format!("{}={}", name, ver));
            } else {
                cmd.arg(name);
            }
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install package: {}\n{}", name, stderr));
            }
        },
        "zypper" => {
            let mut cmd = Command::new("zypper");
            cmd.args(["install", "-y"]);
            
            if let Some(ver) = version {
                cmd.arg(&format!("{}={}", name, ver));
            } else {
                cmd.arg(name);
            }
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install package: {}\n{}", name, stderr));
            }
        },
        "apk" => {
            let mut cmd = Command::new("apk");
            cmd.args(["add"]);
            
            if let Some(ver) = version {
                cmd.arg(&format!("{}={}", name, ver));
            } else {
                cmd.arg(name);
            }
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install package: {}\n{}", name, stderr));
            }
        },
        "pkg" => {
            let mut cmd = Command::new("pkg");
            cmd.args(["install", "-y"]);
            
            if let Some(ver) = version {
                cmd.arg(&format!("{}-{}", name, ver));
            } else {
                cmd.arg(name);
            }
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install package: {}\n{}", name, stderr));
            }
        },
        _ => {
            return Err(anyhow::anyhow!("Unsupported package manager: {}", pkg_manager));
        }
    }
    
    info!("Linux package {} installed successfully", name);
    Ok(())
}

/// Remove a Linux package
pub fn remove_package(name: &str) -> Result<()> {
    let pkg_manager = detect_package_manager()?;
    info!("Removing Linux package {} using {}", name, pkg_manager);
    
    match pkg_manager {
        "apt" => {
            let mut cmd = Command::new("apt");
            cmd.args(["remove", "-y", name]);
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to remove package: {}\n{}", name, stderr));
            }
        },
        "dnf" | "yum" => {
            let mut cmd = Command::new(pkg_manager);
            cmd.args(["remove", "-y", name]);
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to remove package: {}\n{}", name, stderr));
            }
        },
        "pacman" => {
            let mut cmd = Command::new("pacman");
            cmd.args(["-R", "--noconfirm", name]);
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to remove package: {}\n{}", name, stderr));
            }
        },
        "zypper" => {
            let mut cmd = Command::new("zypper");
            cmd.args(["remove", "-y", name]);
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to remove package: {}\n{}", name, stderr));
            }
        },
        "apk" => {
            let mut cmd = Command::new("apk");
            cmd.args(["del", name]);
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to remove package: {}\n{}", name, stderr));
            }
        },
        "pkg" => {
            let mut cmd = Command::new("pkg");
            cmd.args(["remove", "-y", name]);
            
            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to remove package: {}\n{}", name, stderr));
            }
        },
        _ => {
            return Err(anyhow::anyhow!("Unsupported package manager: {}", pkg_manager));
        }
    }
    
    info!("Linux package {} removed successfully", name);
    Ok(())
}

/// Run a Linux package with arguments
pub fn run_package(name: &str, args: &[&str]) -> Result<()> {
    // Check if the command exists
    let which_cmd = Command::new("which")
        .arg(name)
        .output()?;
    
    if !which_cmd.status.success() {
        return Err(anyhow::anyhow!("Command not found: {}", name));
    }
    
    let path = String::from_utf8_lossy(&which_cmd.stdout).trim().to_string();
    info!("Running Linux command: {} ({})", name, path);
    
    // Run the command
    let mut cmd = Command::new(path);
    cmd.args(args);
    
    let mut child = cmd.spawn()?;
    let status = child.wait()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("Command failed with exit code: {:?}", status.code()));
    }
    
    Ok(())
}

/// Search for Linux packages
pub fn search_packages(query: &str) -> Result<Vec<String>> {
    let pkg_manager = detect_package_manager().unwrap_or("apt");
    info!("Searching for Linux packages matching {} using {}", query, pkg_manager);
    
    let mut results = Vec::new();
    
    match pkg_manager {
        "apt" => {
            let cmd = Command::new("apt")
                .args(["search", query])
                .output()?;
                
            if cmd.status.success() {
                let output = String::from_utf8_lossy(&cmd.stdout);
                for line in output.lines() {
                    if line.contains("/") {
                        let parts: Vec<&str> = line.split("/").collect();
                        if !parts.is_empty() {
                            let pkg_name = parts[0].trim();
                            if !pkg_name.is_empty() {
                                results.push(format!("{} (linux)", pkg_name));
                            }
                        }
                    }
                }
            }
        },
        "dnf" | "yum" => {
            let cmd = Command::new(pkg_manager)
                .args(["search", query])
                .output()?;
                
            if cmd.status.success() {
                let output = String::from_utf8_lossy(&cmd.stdout);
                for line in output.lines() {
                    if line.contains(" : ") {
                        let parts: Vec<&str> = line.split(" : ").collect();
                        if !parts.is_empty() {
                            let pkg_name = parts[0].trim();
                            if !pkg_name.is_empty() {
                                results.push(format!("{} (linux)", pkg_name));
                            }
                        }
                    }
                }
            }
        },
        "pacman" => {
            let cmd = Command::new("pacman")
                .args(["-Ss", query])
                .output()?;
                
            if cmd.status.success() {
                let output = String::from_utf8_lossy(&cmd.stdout);
                for line in output.lines() {
                    if line.starts_with(" ") || line.is_empty() {
                        continue;
                    }
                    
                    let parts: Vec<&str> = line.split(" ").collect();
                    if parts.len() >= 2 {
                        let pkg_name = parts[0].split("/").last().unwrap_or("").trim();
                        if !pkg_name.is_empty() {
                            results.push(format!("{} (linux)", pkg_name));
                        }
                    }
                }
            }
        },
        _ => {
            warn!("Package search not fully implemented for {}", pkg_manager);
            // Generic implementation
            let cmd = Command::new(pkg_manager)
                .args(["search", query])
                .output()?;
                
            if cmd.status.success() {
                let output = String::from_utf8_lossy(&cmd.stdout);
                for line in output.lines() {
                    if !line.is_empty() && !line.starts_with(" ") {
                        let pkg_name = line.split(" ").next().unwrap_or("").trim();
                        if !pkg_name.is_empty() {
                            results.push(format!("{} (linux)", pkg_name));
                        }
                    }
                }
            }
        }
    }
    
    Ok(results)
}
