// SentientOS - Zero-Knowledge Operating System
// Main entry point

mod core;
mod cli;
mod zk;
mod matrixbox;
mod linux;
mod boot;
mod gossip;
mod intent;
mod heal;
mod panic;
mod store;

use anyhow::{Result, Context};
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{info, warn, error, debug, Level};

/// Main entry point for SentientOS
fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("SENTIENT_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting SentientOS");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Initialize core subsystems
    core::init()?;
    
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "cli" {
        // CLI mode - handle command directly
        debug!("Running in CLI mode");
        cli::execute_command(args[2..].to_vec())?;
    } else if args.len() > 1 && args[1] == "init" {
        // Initialization mode - bootstrap full system
        info!("Running in initialization mode");
        bootstrap_system()?;
    } else {
        // Interactive mode - start runtime
        info!("Running in interactive mode");
        start_runtime()?;
    }

    info!("SentientOS terminated successfully");
    Ok(())
}

/// Bootstrap the entire system
fn bootstrap_system() -> Result<()> {
    info!("Bootstrapping system...");
    
    // Initialize all subsystems in order
    cli::init().context("Failed to initialize CLI")?;
    zk::init().context("Failed to initialize ZK")?;
    matrixbox::init().context("Failed to initialize MatrixBox")?;
    linux::init().context("Failed to initialize Linux compatibility")?;
    boot::init().context("Failed to initialize Boot")?;
    gossip::init().context("Failed to initialize Gossip")?;
    intent::init().context("Failed to initialize Intent")?;
    heal::init().context("Failed to initialize Heal")?;
    panic::init().context("Failed to initialize Panic")?;
    store::init().context("Failed to initialize ZK-Store")?;
    
    info!("System bootstrap complete");
    Ok(())
}

/// Start the interactive runtime
fn start_runtime() -> Result<()> {
    info!("Starting SentientOS runtime...");
    
    // Initialize minimal set of subsystems
    cli::init()?;
    zk::init()?;
    matrixbox::init()?;
    linux::init()?;
    heal::init()?;
    panic::init()?;
    store::init()?;
    
    // Start interactive shell or service listener here
    // This would typically block until termination
    println!("SentientOS is running. Press Ctrl+C to exit.");
    
    // Wait for termination signal
    wait_for_termination();
    
    // Perform clean shutdown
    shutdown()
}

/// Wait for termination signal
fn wait_for_termination() {
    // In a real implementation, this would wait for a signal
    // For this prototype, we'll just sleep for a moment
    std::thread::sleep(std::time::Duration::from_secs(1));
}

/// Clean shutdown of all subsystems
fn shutdown() -> Result<()> {
    info!("Shutting down SentientOS...");
    
    // Shutdown in reverse order of initialization
    store::shutdown().ok();
    panic::shutdown().ok();
    heal::shutdown().ok();
    intent::shutdown().ok();
    gossip::shutdown().ok();
    boot::shutdown().ok();
    linux::shutdown().ok();
    matrixbox::shutdown().ok();
    zk::shutdown().ok();
    cli::shutdown().ok();
    core::shutdown().ok();
    
    info!("Shutdown complete");
    Ok(())
}
