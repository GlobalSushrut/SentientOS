use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::PathBuf;
use std::fs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use std::thread;
use std::net::{UdpSocket, ToSocketAddrs};

use crate::core::constants;
use super::{PeerStatus, PeerInfo};

// Peer activity timeouts
const PEER_OFFLINE_THRESHOLD: u64 = 120; // seconds
const HEARTBEAT_INTERVAL: u64 = 30; // seconds
const DISCOVERY_INTERVAL: u64 = 300; // seconds

// Global peer tracker
lazy_static::lazy_static! {
    static ref PEER_HEARTBEAT_THREAD: Arc<Mutex<Option<std::thread::JoinHandle<()>>>> = 
        Arc::new(Mutex::new(None));
}

/// Initialize the peers subsystem
pub fn init() -> Result<()> {
    info!("Initializing gossip peers subsystem");
    
    // Create peers directory
    let peers_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("peers");
    
    fs::create_dir_all(&peers_dir)?;
    
    // Start heartbeat thread
    start_heartbeat_thread()?;
    
    info!("Gossip peers subsystem initialized");
    Ok(())
}

/// Shutdown the peers subsystem
pub fn shutdown() -> Result<()> {
    info!("Shutting down gossip peers subsystem");
    
    // Stop heartbeat thread if running
    let mut heartbeat_thread = PEER_HEARTBEAT_THREAD.lock().unwrap();
    if let Some(handle) = heartbeat_thread.take() {
        // Just let it finish naturally - we don't have a way to abort threads in Rust
        debug!("Waiting for heartbeat thread to terminate");
        // We don't want to block shutdown, so we don't join the thread
    }
    
    info!("Gossip peers subsystem shutdown complete");
    Ok(())
}

/// Start the heartbeat thread
fn start_heartbeat_thread() -> Result<()> {
    let mut heartbeat_thread = PEER_HEARTBEAT_THREAD.lock().unwrap();
    
    // If thread is already running, do nothing
    if heartbeat_thread.is_some() {
        return Ok(());
    }
    
    // Start the thread
    let thread_handle = thread::spawn(|| {
        heartbeat_loop();
    });
    
    // Store the handle
    *heartbeat_thread = Some(thread_handle);
    
    debug!("Started peer heartbeat thread");
    Ok(())
}

/// Main heartbeat loop
fn heartbeat_loop() {
    let mut last_heartbeat = 0;
    let mut last_discovery = 0;
    
    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();
        
        // Check if it's time to send heartbeats
        if now - last_heartbeat >= HEARTBEAT_INTERVAL {
            if let Err(e) = send_heartbeats() {
                error!("Error sending heartbeats: {}", e);
            }
            last_heartbeat = now;
        }
        
        // Check if it's time to send discovery
        if now - last_discovery >= DISCOVERY_INTERVAL {
            if let Err(e) = super::protocol::send_discovery_ping() {
                error!("Error sending discovery ping: {}", e);
            }
            last_discovery = now;
        }
        
        // Update peer status based on last seen time
        if let Err(e) = update_peer_statuses() {
            error!("Error updating peer statuses: {}", e);
        }
        
        // Sleep to avoid busy waiting
        thread::sleep(Duration::from_secs(1));
    }
}

/// Send heartbeats to all known peers
fn send_heartbeats() -> Result<()> {
    debug!("Sending heartbeats to peers");
    
    // Get list of peers
    let peers = super::list_peers()?;
    
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for peer in &peers {
        // Skip offline peers
        if peer.status == PeerStatus::Offline {
            continue;
        }
        
        // Create empty heartbeat payload
        let payload = vec![];
        
        // Send heartbeat message
        match super::protocol::send_message(&peer.endpoint, super::protocol::MessageType::Heartbeat, &payload) {
            Ok(_) => {
                success_count += 1;
            },
            Err(e) => {
                failure_count += 1;
                warn!("Failed to send heartbeat to peer {}: {}", peer.id, e);
            }
        }
    }
    
    debug!("Sent heartbeats to {} peers, {} failures", success_count, failure_count);
    Ok(())
}

/// Update peer statuses based on last seen time
fn update_peer_statuses() -> Result<()> {
    // Get list of peers
    let peers = super::list_peers()?;
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();
    
    for peer in &peers {
        // Skip peers that are already offline
        if peer.status == PeerStatus::Offline {
            continue;
        }
        
        // Check if peer is offline based on last seen time
        if now - peer.last_seen > PEER_OFFLINE_THRESHOLD {
            // Mark peer as offline
            super::update_peer_status(&peer.id, PeerStatus::Offline)?;
            debug!("Peer {} marked as offline", peer.id);
        }
    }
    
    Ok(())
}

/// Check peer reachability
pub fn check_peer_reachability(peer_id: &str) -> Result<bool> {
    // Get peer information
    let peers = super::list_peers()?;
    let peer = peers.iter().find(|p| p.id == peer_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown peer: {}", peer_id))?;
    
    // Parse endpoint to socket address
    let addr = peer.endpoint.to_socket_addrs()
        .with_context(|| format!("Invalid endpoint: {}", peer.endpoint))?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not resolve endpoint: {}", peer.endpoint))?;
    
    // Try to reach the peer
    // This is a simple check - we just try to establish a UDP connection
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            match socket.connect(addr) {
                Ok(_) => {
                    // Send a small test packet
                    let test_data = [0u8; 1];
                    match socket.send(&test_data) {
                        Ok(_) => {
                            debug!("Peer {} is reachable", peer_id);
                            Ok(true)
                        },
                        Err(_) => {
                            debug!("Peer {} is unreachable (send failed)", peer_id);
                            Ok(false)
                        }
                    }
                },
                Err(_) => {
                    debug!("Peer {} is unreachable (connect failed)", peer_id);
                    Ok(false)
                }
            }
        },
        Err(e) => {
            warn!("Failed to create socket for reachability check: {}", e);
            Ok(false)
        }
    }
}

/// Load peer information
pub fn load_peer_info(peer_id: &str) -> Result<PeerDetails> {
    let peer_file = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("peers")
        .join(format!("{}.json", peer_id));
    
    if !peer_file.exists() {
        return Err(anyhow::anyhow!("Peer information not found for: {}", peer_id));
    }
    
    // Read and parse peer details
    let peer_json = fs::read_to_string(&peer_file)
        .with_context(|| format!("Failed to read peer file: {}", peer_id))?;
    
    let peer_details: PeerDetails = serde_json::from_str(&peer_json)
        .with_context(|| format!("Failed to parse peer details for: {}", peer_id))?;
    
    Ok(peer_details)
}

/// Save peer information
pub fn save_peer_info(peer_id: &str, details: &PeerDetails) -> Result<()> {
    let peer_file = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("peers")
        .join(format!("{}.json", peer_id));
    
    // Ensure parent directory exists
    if let Some(parent) = peer_file.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Serialize and write
    let peer_json = serde_json::to_string_pretty(details)
        .with_context(|| format!("Failed to serialize peer details for: {}", peer_id))?;
    
    fs::write(&peer_file, peer_json)
        .with_context(|| format!("Failed to write peer file: {}", peer_id))?;
    
    debug!("Saved peer information for: {}", peer_id);
    Ok(())
}

/// Detailed peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDetails {
    /// Peer ID
    pub id: String,
    
    /// Network endpoint
    pub endpoint: String,
    
    /// Peer capabilities
    pub capabilities: Vec<String>,
    
    /// Peer version
    pub version: String,
    
    /// First discovered timestamp
    pub discovered_at: u64,
    
    /// Last connection timestamp
    pub last_connected: u64,
    
    /// Synchronization history
    pub sync_history: Vec<SyncEvent>,
    
    /// Trust level (0-100)
    pub trust_level: u8,
}

/// Synchronization event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    /// Timestamp
    pub timestamp: u64,
    
    /// Type of event
    pub event_type: String,
    
    /// Result status
    pub status: String,
    
    /// Description
    pub description: String,
}
