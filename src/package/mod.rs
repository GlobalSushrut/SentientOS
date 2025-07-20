// SentientOS Universal Package Manager
// Handles multiple package ecosystems including Linux packages, npm, Python, Java, etc.

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::collections::HashMap;
use std::process::Command;
use serde::{Serialize, Deserialize};

use crate::core::constants;
use crate::zk;
use crate::matrixbox;
use crate::store;

pub mod linux;
pub mod npm;
pub mod python;
pub mod java;

// Constants
const PACKAGE_DIR: &str = ".package";
const REGISTRY_FILE: &str = "registry.json";
const CONFIG_FILE: &str = "config.json";

/// Package ecosystem types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Ecosystem {
    /// Native SentientOS packages (uses ZK-Store)
    Native,
    
    /// Linux system packages
    Linux,
    
    /// Node.js packages (npm)
    Npm,
    
    /// Python packages (pip)
    Python,
    
    /// Java packages (maven)
    Java,
    
    /// Rust packages (cargo)
    Rust,
    
    /// Go packages
    Go,
    
    /// Other ecosystem
    Other(String),
}

/// Installed package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    /// Package name
    pub name: String,
    
    /// Package version
    pub version: String,
    
    /// Package ecosystem
    pub ecosystem: Ecosystem,
    
    /// Installation path
    pub path: String,
    
    /// Container ID if running in MatrixBox
    pub container_id: Option<String>,
    
    /// Installation timestamp
    pub installed_at: u64,
    
    /// Configuration options
    pub config: HashMap<String, String>,
}

/// Package registry to track installed packages across ecosystems
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageRegistry {
    /// Last updated timestamp
    pub last_updated: u64,
    
    /// Installed packages
    pub packages: HashMap<String, InstalledPackage>,
}

/// Package manager configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Default installation paths for ecosystems
    pub ecosystem_paths: HashMap<String, String>,
    
    /// Whether to verify packages with ZK proofs when possible
    pub zk_verify: bool,
    
    /// Whether to isolate packages in MatrixBox containers
    pub isolate: bool,
    
    /// Global environment variables
    pub env_vars: HashMap<String, String>,
}

/// Initialize the package manager
pub fn init() -> Result<()> {
    info!("Initializing Universal Package Manager");
    
    // Create package directories
    let package_dir = PathBuf::from(constants::ROOT_DIR).join(PACKAGE_DIR);
    fs::create_dir_all(&package_dir)?;
    
    // Initialize registry if it doesn't exist
    let registry_path = package_dir.join(REGISTRY_FILE);
    if !registry_path.exists() {
        let empty_registry = PackageRegistry {
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            packages: HashMap::new(),
        };
        
        let registry_json = serde_json::to_string_pretty(&empty_registry)?;
        fs::write(&registry_path, registry_json)?;
    }
    
    // Initialize config if it doesn't exist
    let config_path = package_dir.join(CONFIG_FILE);
    if !config_path.exists() {
        let default_config = PackageConfig {
            ecosystem_paths: [
                ("Native".to_string(), format!("{}/packages", constants::ROOT_DIR)),
                ("Linux".to_string(), "/usr/bin".to_string()),
                ("Npm".to_string(), format!("{}/packages/npm", constants::ROOT_DIR)),
                ("Python".to_string(), format!("{}/packages/python", constants::ROOT_DIR)),
                ("Java".to_string(), format!("{}/packages/java", constants::ROOT_DIR)),
                ("Rust".to_string(), format!("{}/packages/rust", constants::ROOT_DIR)),
                ("Go".to_string(), format!("{}/packages/go", constants::ROOT_DIR)),
            ].iter().cloned().collect(),
            zk_verify: true,
            isolate: true,
            env_vars: HashMap::new(),
        };
        
        let config_json = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_path, config_json)?;
    }
    
    // Ensure ecosystem directories exist
    let config = load_config()?;
    for (_, path) in config.ecosystem_paths {
        fs::create_dir_all(path)?;
    }
    
    info!("Universal Package Manager initialized successfully");
    Ok(())
}

/// Load package manager configuration
pub fn load_config() -> Result<PackageConfig> {
    let package_dir = PathBuf::from(constants::ROOT_DIR).join(PACKAGE_DIR);
    let config_path = package_dir.join(CONFIG_FILE);
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Package manager not initialized"));
    }
    
    let config_data = fs::read_to_string(&config_path)?;
    let config: PackageConfig = serde_json::from_str(&config_data)?;
    
    Ok(config)
}

/// Load package registry
pub fn load_registry() -> Result<PackageRegistry> {
    let package_dir = PathBuf::from(constants::ROOT_DIR).join(PACKAGE_DIR);
    let registry_path = package_dir.join(REGISTRY_FILE);
    
    if !registry_path.exists() {
        return Err(anyhow::anyhow!("Package registry not initialized"));
    }
    
    let registry_data = fs::read_to_string(&registry_path)?;
    let registry: PackageRegistry = serde_json::from_str(&registry_data)?;
    
    Ok(registry)
}

/// Save package registry
fn save_registry(registry: &PackageRegistry) -> Result<()> {
    let package_dir = PathBuf::from(constants::ROOT_DIR).join(PACKAGE_DIR);
    let registry_path = package_dir.join(REGISTRY_FILE);
    
    let registry_json = serde_json::to_string_pretty(&registry)?;
    fs::write(&registry_path, registry_json)?;
    
    Ok(())
}

/// Install a package from any supported ecosystem
pub fn install_package(name: &str, ecosystem: Ecosystem, version: Option<&str>) -> Result<()> {
    info!("Installing package: {} from {:?} ecosystem", name, ecosystem);
    
    // Check if already installed
    let mut registry = load_registry()?;
    let config = load_config()?;
    
    let full_name = match &ecosystem {
        Ecosystem::Native => name.to_string(),
        Ecosystem::Linux => format!("linux:{}", name),
        Ecosystem::Npm => format!("npm:{}", name),
        Ecosystem::Python => format!("python:{}", name),
        Ecosystem::Java => format!("java:{}", name),
        Ecosystem::Rust => format!("rust:{}", name),
        Ecosystem::Go => format!("go:{}", name),
        Ecosystem::Other(eco) => format!("{}:{}", eco, name),
    };
    
    if registry.packages.contains_key(&full_name) {
        if let Some(ver) = version {
            if registry.packages.get(&full_name).unwrap().version == ver {
                info!("Package {} already installed", full_name);
                return Ok(());
            }
        } else {
            info!("Package {} already installed", full_name);
            return Ok(());
        }
    }
    
    // Install based on ecosystem
    match ecosystem {
        Ecosystem::Native => {
            // Use existing ZK-Store for native packages
            store::install_package(name)?;
        },
        Ecosystem::Linux => {
            linux::install_package(name, version)?;
        },
        Ecosystem::Npm => {
            npm::install_package(name, version)?;
        },
        Ecosystem::Python => {
            python::install_package(name, version)?;
        },
        Ecosystem::Java => {
            java::install_package(name, version)?;
        },
        Ecosystem::Rust => {
            // Use cargo to install Rust packages
            let mut cmd = Command::new("cargo");
            cmd.arg("install");
            cmd.arg(name);
            if let Some(ver) = version {
                cmd.args(["--version", ver]);
            }
            
            let output = cmd.output()?;
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to install Rust package: {}", name));
            }
        },
        Ecosystem::Go => {
            // Use go get to install Go packages
            let mut cmd = Command::new("go");
            cmd.arg("install");
            
            let package_spec = if let Some(ver) = version {
                format!("{}@{}", name, ver)
            } else {
                name.to_string()
            };
            
            cmd.arg(&package_spec);
            
            let output = cmd.output()?;
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to install Go package: {}", name));
            }
        },
        Ecosystem::Other(eco) => {
            return Err(anyhow::anyhow!("Unsupported ecosystem: {}", eco));
        }
    }
    
    // Add to registry
    let version_str = match version {
        Some(v) => v.to_string(),
        None => "latest".to_string(),
    };
    
    let ecosystem_path = match ecosystem {
        Ecosystem::Native => config.ecosystem_paths.get("Native"),
        Ecosystem::Linux => config.ecosystem_paths.get("Linux"),
        Ecosystem::Npm => config.ecosystem_paths.get("Npm"),
        Ecosystem::Python => config.ecosystem_paths.get("Python"),
        Ecosystem::Java => config.ecosystem_paths.get("Java"),
        Ecosystem::Rust => config.ecosystem_paths.get("Rust"),
        Ecosystem::Go => config.ecosystem_paths.get("Go"),
        Ecosystem::Other(eco) => config.ecosystem_paths.get(&eco),
    };
    
    let path = ecosystem_path
        .cloned()
        .unwrap_or_else(|| format!("{}/packages", constants::ROOT_DIR));
    
    let installed_pkg = InstalledPackage {
        name: name.to_string(),
        version: version_str,
        ecosystem: ecosystem.clone(),
        path,
        container_id: None,
        installed_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        config: HashMap::new(),
    };
    
    registry.packages.insert(full_name.clone(), installed_pkg);
    registry.last_updated = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    save_registry(&registry)?;
    
    info!("Package {} installed successfully", full_name);
    Ok(())
}

/// Remove an installed package
pub fn remove_package(name: &str, ecosystem: Option<Ecosystem>) -> Result<()> {
    let mut registry = load_registry()?;
    
    // If ecosystem is specified, create full name
    let package_key = if let Some(eco) = ecosystem {
        match eco {
            Ecosystem::Native => name.to_string(),
            Ecosystem::Linux => format!("linux:{}", name),
            Ecosystem::Npm => format!("npm:{}", name),
            Ecosystem::Python => format!("python:{}", name),
            Ecosystem::Java => format!("java:{}", name),
            Ecosystem::Rust => format!("rust:{}", name),
            Ecosystem::Go => format!("go:{}", name),
            Ecosystem::Other(eco_name) => format!("{}:{}", eco_name, name),
        }
    } else {
        // Try to find by name only
        let matches: Vec<_> = registry.packages.keys()
            .filter(|k| k.ends_with(&format!(":{}", name)) || *k == name)
            .cloned()
            .collect();
            
        if matches.is_empty() {
            return Err(anyhow::anyhow!("Package not found: {}", name));
        } else if matches.len() > 1 {
            return Err(anyhow::anyhow!("Multiple packages found with name {}, please specify ecosystem", name));
        }
        
        matches[0].clone()
    };
    
    // Check if package exists
    if !registry.packages.contains_key(&package_key) {
        return Err(anyhow::anyhow!("Package not installed: {}", package_key));
    }
    
    let package = registry.packages.remove(&package_key).unwrap();
    
    // Uninstall based on ecosystem
    match package.ecosystem {
        Ecosystem::Native => {
            store::remove_package(name)?;
        },
        Ecosystem::Linux => {
            linux::remove_package(name)?;
        },
        Ecosystem::Npm => {
            npm::remove_package(name)?;
        },
        Ecosystem::Python => {
            python::remove_package(name)?;
        },
        Ecosystem::Java => {
            java::remove_package(name)?;
        },
        Ecosystem::Rust => {
            // Use cargo to uninstall Rust packages
            let mut cmd = Command::new("cargo");
            cmd.args(["uninstall", name]);
            
            let output = cmd.output()?;
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to uninstall Rust package: {}", name));
            }
        },
        Ecosystem::Go => {
            // Go doesn't have a direct uninstall command, just remove the binary
            let go_bin = std::env::var("GOBIN").unwrap_or_else(|_| format!("{}/go/bin", std::env::var("HOME").unwrap_or_default()));
            let bin_path = PathBuf::from(go_bin).join(name);
            if bin_path.exists() {
                fs::remove_file(bin_path)?;
            }
        },
        Ecosystem::Other(eco) => {
            warn!("No uninstall handler for ecosystem {}, just removing from registry", eco);
        }
    }
    
    // Update registry
    registry.last_updated = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    save_registry(&registry)?;
    
    info!("Package {} removed successfully", package_key);
    Ok(())
}

/// List installed packages, optionally filtered by ecosystem
pub fn list_packages(ecosystem: Option<Ecosystem>) -> Result<Vec<InstalledPackage>> {
    let registry = load_registry()?;
    
    let packages = if let Some(eco) = ecosystem {
        registry.packages.values()
            .filter(|p| p.ecosystem == eco)
            .cloned()
            .collect()
    } else {
        registry.packages.values().cloned().collect()
    };
    
    Ok(packages)
}

/// Run a package with optional arguments
pub fn run_package(name: &str, ecosystem: Option<Ecosystem>, args: &[&str]) -> Result<()> {
    let registry = load_registry()?;
    let config = load_config()?;
    
    // Find the package
    let package = if let Some(eco) = ecosystem {
        let full_name = match eco {
            Ecosystem::Native => name.to_string(),
            Ecosystem::Linux => format!("linux:{}", name),
            Ecosystem::Npm => format!("npm:{}", name),
            Ecosystem::Python => format!("python:{}", name),
            Ecosystem::Java => format!("java:{}", name),
            Ecosystem::Rust => format!("rust:{}", name),
            Ecosystem::Go => format!("go:{}", name),
            Ecosystem::Other(eco_name) => format!("{}:{}", eco_name, name),
        };
        
        registry.packages.get(&full_name).cloned()
    } else {
        // Try to find by name only
        let matches: Vec<_> = registry.packages.iter()
            .filter(|(k, _)| k.ends_with(&format!(":{}", name)) || *k == name)
            .map(|(_, v)| v)
            .collect();
            
        if matches.is_empty() {
            None
        } else if matches.len() > 1 {
            return Err(anyhow::anyhow!("Multiple packages found with name {}, please specify ecosystem", name));
        } else {
            matches[0].cloned()
        }
    };
    
    if let Some(pkg) = package {
        // Run based on ecosystem
        match pkg.ecosystem {
            Ecosystem::Native => {
                // Run in MatrixBox container if isolate is enabled
                if config.isolate {
                    let container_id = pkg.container_id.clone().unwrap_or_else(|| name.to_string());
                    matrixbox::run_container(&container_id, args)?;
                } else {
                    // Run directly
                    let bin_path = PathBuf::from(&pkg.path).join(name);
                    let mut cmd = Command::new(bin_path);
                    cmd.args(args);
                    
                    let mut child = cmd.spawn()?;
                    child.wait()?;
                }
            },
            Ecosystem::Linux => {
                linux::run_package(name, args)?;
            },
            Ecosystem::Npm => {
                npm::run_package(name, args)?;
            },
            Ecosystem::Python => {
                python::run_package(name, args)?;
            },
            Ecosystem::Java => {
                java::run_package(name, args)?;
            },
            Ecosystem::Rust => {
                // Run Rust binary directly
                let mut cmd = Command::new(name);
                cmd.args(args);
                
                let mut child = cmd.spawn()?;
                child.wait()?;
            },
            Ecosystem::Go => {
                // Run Go binary directly
                let mut cmd = Command::new(name);
                cmd.args(args);
                
                let mut child = cmd.spawn()?;
                child.wait()?;
            },
            Ecosystem::Other(eco) => {
                return Err(anyhow::anyhow!("Running packages from ecosystem {} not supported", eco));
            }
        }
        
        Ok(())
    } else {
        Err(anyhow::anyhow!("Package not found: {}", name))
    }
}

/// Search for packages across ecosystems
pub fn search_packages(query: &str, ecosystem: Option<Ecosystem>) -> Result<Vec<String>> {
    info!("Searching for packages matching: {}", query);
    
    let mut results = Vec::new();
    
    match ecosystem {
        Some(Ecosystem::Native) => {
            // Search in ZK-Store
            let packages = store::search_packages(query)?;
            for pkg in packages {
                results.push(format!("{} (native) - {}", pkg.name, pkg.description));
            }
        },
        Some(Ecosystem::Linux) => {
            // Search Linux packages
            results.extend(linux::search_packages(query)?);
        },
        Some(Ecosystem::Npm) => {
            // Search npm packages
            results.extend(npm::search_packages(query)?);
        },
        Some(Ecosystem::Python) => {
            // Search Python packages
            results.extend(python::search_packages(query)?);
        },
        Some(Ecosystem::Java) => {
            // Search Java packages
            results.extend(java::search_packages(query)?);
        },
        Some(Ecosystem::Rust) => {
            // Search Rust crates
            // This would use the crates.io API in a real implementation
            info!("Rust crate search not implemented in prototype");
        },
        Some(Ecosystem::Go) => {
            // Search Go packages
            // This would use the pkg.go.dev API in a real implementation
            info!("Go package search not implemented in prototype");
        },
        Some(Ecosystem::Other(eco)) => {
            return Err(anyhow::anyhow!("Search not supported for ecosystem: {}", eco));
        },
        None => {
            // Search across all ecosystems
            let packages = store::search_packages(query)?;
            for pkg in packages {
                results.push(format!("{} (native) - {}", pkg.name, pkg.description));
            }
            
            results.extend(linux::search_packages(query)?);
            results.extend(npm::search_packages(query)?);
            results.extend(python::search_packages(query)?);
            results.extend(java::search_packages(query)?);
        }
    }
    
    Ok(results)
}

/// Create an application container from installed packages
pub fn create_app(name: &str, packages: &[&str], icon: Option<&str>, desktop_entry: bool) -> Result<()> {
    info!("Creating application: {}", name);
    
    let registry = load_registry()?;
    
    // Verify all packages exist
    for pkg_name in packages {
        let found = registry.packages.iter().any(|(k, _)| {
            k == pkg_name || k.ends_with(&format!(":{}", pkg_name))
        });
        
        if !found {
            return Err(anyhow::anyhow!("Package not found: {}", pkg_name));
        }
    }
    
    // Create MatrixBox container for the app
    let container_config = matrixbox::ContainerConfig {
        name: name.to_string(),
        description: Some(format!("Application container for {}", name)),
        version: Some("1.0".to_string()),
        author: None,
        ..Default::default()
    };
    
    // Create app directory
    let app_dir = PathBuf::from(constants::ROOT_DIR).join("apps").join(name);
    fs::create_dir_all(&app_dir)?;
    
    // Create app metadata
    let metadata = serde_json::json!({
        "name": name,
        "packages": packages,
        "created_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "icon": icon,
    });
    
    let metadata_path = app_dir.join("app.json");
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;
    
    // Create app runner script
    let runner_script = format!(
        r#"#!/bin/bash
# Auto-generated runner script for {}
echo "Starting application: {}"
"#,
        name, name
    );
    
    let runner_path = app_dir.join("run.sh");
    fs::write(&runner_path, runner_script)?;
    fs::set_permissions(&runner_path, std::fs::Permissions::from_mode(0o755))?;
    
    // Create desktop entry if requested
    if desktop_entry {
        let desktop_dir = PathBuf::from(format!("{}/.local/share/applications", std::env::var("HOME").unwrap_or_default()));
        fs::create_dir_all(&desktop_dir)?;
        
        let icon_path = if let Some(icon_file) = icon {
            let dest_icon = app_dir.join("icon.png");
            fs::copy(icon_file, &dest_icon)?;
            dest_icon.to_string_lossy().to_string()
        } else {
            "/usr/share/icons/hicolor/256x256/apps/applications-system.png".to_string()
        };
        
        let desktop_entry = format!(
            r#"[Desktop Entry]
Type=Application
Name={}
Exec={}
Icon={}
Comment=SentientOS Application
Terminal=false
Categories=Utility;
"#,
            name, runner_path.to_string_lossy(), icon_path
        );
        
        let desktop_file = desktop_dir.join(format!("sentientos-{}.desktop", name));
        fs::write(&desktop_file, desktop_entry)?;
    }
    
    // Create MatrixBox container
    matrixbox::create_container(&app_dir, container_config)?;
    
    info!("Application {} created successfully", name);
    Ok(())
}

/// Update a package to the latest version
pub fn update_package(name: &str, ecosystem: Option<Ecosystem>) -> Result<()> {
    let registry = load_registry()?;
    
    // Find the package
    let package = if let Some(eco) = ecosystem {
        let full_name = match eco {
            Ecosystem::Native => name.to_string(),
            Ecosystem::Linux => format!("linux:{}", name),
            Ecosystem::Npm => format!("npm:{}", name),
            Ecosystem::Python => format!("python:{}", name),
            Ecosystem::Java => format!("java:{}", name),
            Ecosystem::Rust => format!("rust:{}", name),
            Ecosystem::Go => format!("go:{}", name),
            Ecosystem::Other(eco_name) => format!("{}:{}", eco_name, name),
        };
        
        registry.packages.get(&full_name).cloned()
    } else {
        // Try to find by name only
        let matches: Vec<_> = registry.packages.iter()
            .filter(|(k, _)| k.ends_with(&format!(":{}", name)) || *k == name)
            .map(|(_, v)| v)
            .collect();
            
        if matches.is_empty() {
            None
        } else if matches.len() > 1 {
            return Err(anyhow::anyhow!("Multiple packages found with name {}, please specify ecosystem", name));
        } else {
            matches[0].cloned()
        }
    };
    
    if let Some(pkg) = package {
        // Remove and reinstall the package
        remove_package(name, Some(pkg.ecosystem.clone()))?;
        install_package(name, pkg.ecosystem, None)?;
        
        info!("Package {} updated successfully", name);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Package not found: {}", name))
    }
}
