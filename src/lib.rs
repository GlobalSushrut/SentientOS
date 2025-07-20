// SentientOS - A next-generation, ultra-lightweight, ZK-proofed, dynamic OS
// Core library file

pub mod core;
pub mod zk;
pub mod matrixbox;
pub mod runtime;
pub mod linux;
pub mod auth;
pub mod gossip;
pub mod heal;
pub mod boot;
pub mod panic;
pub mod intent;
pub mod filesystem;
pub mod cli;
pub mod network;
pub mod store;
pub mod package;

/// Version of Sentinent OS
pub const VERSION: &str = "0.1.0";

/// Initialize the Sentinent OS runtime
pub fn init(zk_enabled: bool) -> anyhow::Result<()> {
    // Initialize logging
    tracing::info!("Initializing Sentinent OS v{} (ZK mode: {})", 
        VERSION, 
        if zk_enabled { "enabled" } else { "disabled" }
    );
    
    // Initialize filesystem structure first
    filesystem::init()?;
    
    // Initialize core directories
    core::fs::ensure_directories()?;
    
    // Initialize the boot subsystem for hardware setup
    boot::init()?;
    
    // Initialize panic system early for fault tolerance
    panic::init()?;
    
    // Initialize the runtime
    runtime::init(zk_enabled)?;
    
    // Initialize auth system
    auth::init()?;
    
    // Initialize MatrixBox container runtime and WASM runtime
    matrixbox::init()?;
    
    // Initialize Linux compatibility layer if needed
    linux::init()?;
    
    // Initialize healing subsystem
    heal::init()?;
    
    // Initialize network subsystem
    network::init()?;
    
    // Initialize gossip synchronization system
    gossip::init()?;
    
    // Initialize ZK system if enabled
    if zk_enabled {
        zk::init()?;
    } else {
        tracing::info!("ZK system disabled, running in trace-only mode");
    }
    
    // Initialize developer intent system
    intent::init()?;
    
    // Initialize store subsystem
    store::init()?;
    
    // Initialize package manager
    package::init()?;
    
    // Initialize CLI interface
    cli::init()?;
    
    // Verify boot integrity after all systems initialized
    let boot_integrity = boot::verify_integrity()?;
    if !boot_integrity {
        tracing::warn!("Boot integrity verification failed - system may be compromised");
    } else {
        tracing::info!("Boot integrity verified successfully");
    }
    
    tracing::info!("Sentinent OS initialized successfully");
    Ok(())
}

/// Shutdown the Sentinent OS runtime
pub fn shutdown() -> anyhow::Result<()> {
    tracing::info!("Shutting down Sentinent OS");
    
    // Shutdown components in reverse order of initialization
    cli::shutdown()?;
    package::shutdown()?;
    store::shutdown()?;
    intent::shutdown()?;
    zk::shutdown()?;
    gossip::shutdown()?;
    network::shutdown()?;
    heal::shutdown()?;
    linux::shutdown()?;
    matrixbox::shutdown()?;
    auth::shutdown()?;
    runtime::shutdown()?;
    panic::shutdown()?;
    boot::shutdown()?; // Shutdown boot subsystem last
    
    tracing::info!("Sentinent OS shutdown complete");
    Ok(())
}
