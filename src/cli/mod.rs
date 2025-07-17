// SentientOS CLI Module
// Implements the sentctl command-line interface

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn, error, debug};
use std::path::{Path, PathBuf};

use crate::matrixbox;
use crate::zk;
use crate::boot;
use crate::core::constants;
use crate::linux;
use crate::store;

/// Initialize the CLI module
pub fn init() -> Result<()> {
    info!("Initializing CLI module");
    
    // Create CLI directories
    let cli_dir = PathBuf::from(constants::ROOT_DIR).join(".cli");
    std::fs::create_dir_all(&cli_dir)?;
    
    info!("CLI module initialized successfully");
    Ok(())
}

/// Shutdown the CLI module
pub fn shutdown() -> Result<()> {
    info!("Shutting down CLI module");
    info!("CLI module shutdown complete");
    Ok(())
}

/// Parse and execute CLI commands
pub fn execute_command(args: Vec<String>) -> Result<()> {
    let cli = Cli::parse_from(args);
    
    match &cli.command {
        Commands::Init { zk_enabled } => {
            info!("Initializing system with ZK: {}", zk_enabled);
            // System initialization logic
            // This would typically be called during boot, not from CLI
            Ok(())
        }
        Commands::ZkVerify {} => {
            info!("Verifying ZK proof chains across system");
            // Implement full system ZK verification
            Ok(())
        }
        Commands::Rollback { target } => {
            info!("Rolling back system to: {}", target);
            crate::heal::rollback_system(target)?;
            Ok(())
        }
        Commands::IsoBuild { output } => {
            info!("Building bootable OS image to: {}", output);
            boot::create_bootable_image(output)?;
            Ok(())
        }
        Commands::Boot { zero } => {
            info!("Booting system with zero mode: {}", zero);
            // This would typically not be called from CLI
            Ok(())
        }
        Commands::TsoRun { container_path } => {
            info!("Running TSO container: {}", container_path);
            matrixbox::run_container(container_path)?;
            Ok(())
        }
        Commands::MatrixBox { command } => {
            match command {
                MatrixBoxCommands::Ls {} => {
                    info!("Listing MatrixBox containers");
                    let containers = matrixbox::list_containers()?;
                    for container in containers {
                        println!("{}: {}", container.id, container.name);
                    }
                }
                MatrixBoxCommands::Rm { id } => {
                    info!("Removing MatrixBox container: {}", id);
                    matrixbox::remove_container(id)?;
                }
            }
            Ok(())
        }
        Commands::Contract { command } => {
            match command {
                ContractCommands::Reload { path } => {
                    info!("Reloading ZK contract: {}", path);
                    let contract = zk::load_contract(path)?;
                    // Implement hot reload logic
                }
                ContractCommands::Verify { path } => {
                    info!("Verifying contract: {}", path);
                    let contract = zk::load_contract(path)?;
                    let result = zk::verify_contract(&contract)?;
                    println!("Contract verification: {}", if result { "PASSED" } else { "FAILED" });
                }
            }
            Ok(())
        }
        Commands::Linux { command } => {
            info!("Executing Linux compatibility command");
            linux::cli::handle_command(command)
        }
        Commands::Store { command } => {
            match command {
                StoreCommands::Install { name } => {
                    info!("Installing package: {}", name);
                    store::install_package(&name)?;
                }
                StoreCommands::Remove { name } => {
                    info!("Removing package: {}", name);
                    store::remove_package(&name)?;
                }
                StoreCommands::List {} => {
                    info!("Listing installed packages");
                    let packages = store::list_installed_packages()?;
                    if packages.is_empty() {
                        println!("No packages installed");
                    } else {
                        for package in packages {
                            println!("{}", package);
                        }
                    }
                }
                StoreCommands::Search { query } => {
                    info!("Searching for packages: {}", query);
                    let packages = store::search_packages(&query)?;
                    if packages.is_empty() {
                        println!("No packages found matching: {}", query);
                    } else {
                        for package in packages {
                            println!("{} ({}): {}", package.name, package.version, package.description);
                        }
                    }
                }
                StoreCommands::Info { name } => {
                    info!("Showing package info: {}", name);
                    let package = store::show_package_details(&name)?;
                    match package {
                        Some(pkg) => {
                            println!("Package: {}", pkg.name);
                            println!("Version: {}", pkg.version);
                            println!("Description: {}", pkg.description);
                            println!("Author: {}", pkg.author);
                            println!("License: {}", pkg.license);
                            println!("Dependencies: {:?}", pkg.dependencies);
                        }
                        None => println!("Package not found: {}", name)
                    }
                }
                StoreCommands::Update {} => {
                    info!("Updating package index");
                    store::update_index()?;
                    println!("Package index updated successfully");
                }
                StoreCommands::Verify { name } => {
                    info!("Verifying package integrity: {}", name);
                    let result = store::verify_package(&name)?;
                    println!("Package integrity: {}", if result { "VALID" } else { "INVALID" });
                }
            }
            Ok(())
        }
        Commands::Heal { command } => {
            match command {
                HealCommands::Container { id } => {
                    info!("Healing container: {}", id);
                    crate::heal::heal_container(id)?;
                }
                HealCommands::Boot {} => {
                    info!("Healing boot subsystem");
                    crate::heal::heal_boot()?;
                }
            }
            Ok(())
        }
        Commands::Panic { command } => {
            match command {
                PanicCommands::Recover {} => {
                    info!("Recovering from panic state");
                    crate::panic::recover()?;
                }
                PanicCommands::Report { output } => {
                    info!("Generating crash report to: {}", output);
                    crate::panic::generate_report(output)?;
                }
            }
            Ok(())
        }
        Commands::Gossip { command } => {
            match command {
                GossipCommands::Enable {} => {
                    info!("Enabling gossip trace sync");
                    crate::gossip::enable_sync()?;
                }
                GossipCommands::Pull { peer } => {
                    info!("Pulling runtime trace from peer: {}", peer);
                    crate::gossip::pull_from_peer(peer)?;
                }
                GossipCommands::VerifyTrace {} => {
                    info!("Cross-validating trace integrity with peers");
                    crate::gossip::verify_trace()?;
                }
            }
            Ok(())
        }
        Commands::Intent { command } => {
            match command {
                IntentCommands::Record {} => {
                    info!("Starting intent recording session");
                    crate::intent::start_recording()?;
                }
                IntentCommands::Stop {} => {
                    info!("Stopping intent recording session");
                    crate::intent::stop_recording()?;
                }
                IntentCommands::Replay { session } => {
                    info!("Replaying intent session: {}", session);
                    crate::intent::replay_session(session)?;
                }
            }
            Ok(())
        }
    }
}

/// CLI command definition using clap
#[derive(Parser)]
#[clap(name = "sentctl")]
#[clap(about = "SentientOS Command Line Interface", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize and bootstrap the runtime
    Init {
        /// Enable ZK proof enforcement
        #[clap(long, default_value = "true")]
        zk_enabled: bool,
    },
    
    /// Verify full ZK proof chains across system
    ZkVerify {},
    
    /// Rollback to previous system state
    Rollback {
        /// Target state to rollback to
        #[clap(default_value = "last-known-good")]
        target: String,
    },
    
    /// Build bootable OS image
    IsoBuild {
        /// Output path for the image
        #[clap(default_value = "sentientos.iso")]
        output: String,
    },
    
    /// Boot into system (normally not called directly)
    Boot {
        /// Boot into minimal zero-mode runtime
        #[clap(long)]
        zero: bool,
    },
    
    /// Execute container inside MatrixBox runtime
    TsoRun {
        /// Path to the TSO container
        container_path: String,
    },
    
    /// MatrixBox container operations
    MatrixBox {
        #[clap(subcommand)]
        command: MatrixBoxCommands,
    },
    
    /// Contract management
    Contract {
        #[clap(subcommand)]
        command: ContractCommands,
    },
    
    /// Healing and recovery commands
    Heal {
        #[clap(subcommand)]
        command: HealCommands,
    },
    
    /// Panic recovery system
    Panic {
        #[clap(subcommand)]
        command: PanicCommands,
    },
    
    /// Multi-device sync and gossip
    Gossip {
        #[clap(subcommand)]
        command: GossipCommands,
    },
    
    /// Developer intent recording and replay
    Intent {
        #[clap(subcommand)]
        command: IntentCommands,
    },
    
    /// Linux compatibility layer commands
    Linux {
        #[clap(subcommand)]
        command: linux::LinuxCommands,
    },
    
    /// ZK-Store package manager commands
    Store {
        #[clap(subcommand)]
        command: StoreCommands,
    },
}

#[derive(Subcommand)]
enum MatrixBoxCommands {
    /// List all running MatrixBox containers
    Ls {},
    
    /// Remove container from MatrixBox registry
    Rm {
        /// Container ID to remove
        id: String,
    },
}

#[derive(Subcommand)]
enum ContractCommands {
    /// Hot-reload ZK contract without reboot
    Reload {
        /// Path to contract
        path: String,
    },
    
    /// Verify contract validity and execution
    Verify {
        /// Path to contract
        path: String,
    },
}

#[derive(Subcommand)]
enum HealCommands {
    /// Auto-recover container from last good state
    Container {
        /// Container ID to heal
        id: String,
    },
    
    /// Rebuild kernel space from last clean .boot
    Boot {},
}

#[derive(Subcommand)]
enum PanicCommands {
    /// Recover from panic state using fallback
    Recover {},
    
    /// Generate crash report from panic logs
    Report {
        /// Output path for report
        #[clap(default_value = "crash_report.json")]
        output: String,
    },
}

#[derive(Subcommand)]
enum GossipCommands {
    /// Enable trace sync between devices
    Enable {},
    
    /// Pull runtime trace from peer device
    Pull {
        /// Peer ID to pull from
        peer: String,
    },
    
    /// Cross-validate trace integrity with peers
    VerifyTrace {},
}

#[derive(Subcommand)]
enum IntentCommands {
    /// Start recording developer intent session
    Record {},
    
    /// Stop recording developer intent session
    Stop {},
    
    /// Replay recorded session for debugging
    Replay {
        /// Session ID to replay
        session: String,
    },
}

#[derive(Subcommand)]
enum StoreCommands {
    /// Install package from ZK-Store
    Install {
        /// Package name to install
        name: String,
    },
    
    /// Remove installed package
    Remove {
        /// Package name to remove
        name: String,
    },
    
    /// List installed packages
    List {},
    
    /// Search for packages in the store
    Search {
        /// Search query
        query: String,
    },
    
    /// Show details for a package
    Info {
        /// Package name
        name: String,
    },
    
    /// Update package index
    Update {},
    
    /// Verify package integrity
    Verify {
        /// Package name to verify
        name: String,
    },
}
