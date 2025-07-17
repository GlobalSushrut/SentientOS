// SentientOS Gossip Synchronization System
// Handles state synchronization and peer-to-peer communication

pub mod protocol;
pub mod peers;
pub mod sync;
pub mod verify;

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use crate::core::constants;

// Global peer registry
lazy_static::lazy_static! {
    static ref PEER_REGISTRY: Arc<Mutex<PeerRegistry>> = 
        Arc::new(Mutex::new(PeerRegistry::new()));
}

/// Initialize the gossip synchronization system
pub fn init() -> Result<()> {
    info!("Initializing SentientOS gossip system");
    
    // Create gossip system directories
    let gossip_dir = PathBuf::from(constants::ROOT_DIR).join(".gossip");
    fs::create_dir_all(&gossip_dir)?;
    
    let peers_dir = gossip_dir.join("peers");
    fs::create_dir_all(&peers_dir)?;
    
    let sync_dir = gossip_dir.join("sync");
    fs::create_dir_all(&sync_dir)?;
    
    let verify_dir = gossip_dir.join("verify");
    fs::create_dir_all(&verify_dir)?;
    
    let archive_dir = gossip_dir.join("archive");
    fs::create_dir_all(&archive_dir)?;
    
    // Initialize components
    protocol::init()?;
    peers::init()?;
    sync::init()?;
    verify::init()?;
    
    // Load peer registry from disk
    load_peer_registry()?;
    
    info!("SentientOS gossip system initialized successfully");
    Ok(())
}

/// Shutdown the gossip synchronization system
pub fn shutdown() -> Result<()> {
    info!("Shutting down SentientOS gossip system");
    
    // Save peer registry before shutdown
    save_peer_registry()?;
    
    // Shutdown components in reverse order
    verify::shutdown()?;
    sync::shutdown()?;
    peers::shutdown()?;
    protocol::shutdown()?;
    
    info!("SentientOS gossip system shutdown complete");
    Ok(())
}

/// Add a new peer to the gossip network
pub fn add_peer(peer_id: &str, endpoint: &str) -> Result<()> {
    info!("Adding peer to gossip network: {}", peer_id);
    
    let mut registry = PEER_REGISTRY.lock().unwrap();
    
    // Create the peer
    let peer = Peer {
        id: peer_id.to_string(),
        endpoint: endpoint.to_string(),
        last_seen: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        status: PeerStatus::Unknown,
        sync_status: HashMap::new(),
    };
    
    // Add to registry
    registry.peers.insert(peer_id.to_string(), peer);
    
    // Persist to disk
    save_peer_registry()?;
    
    // Notify the peer synchronization system
    sync::peer_added(peer_id, endpoint)?;
    
    info!("Peer added successfully: {}", peer_id);
    Ok(())
}

/// Remove a peer from the gossip network
pub fn remove_peer(peer_id: &str) -> Result<()> {
    info!("Removing peer from gossip network: {}", peer_id);
    
    let mut registry = PEER_REGISTRY.lock().unwrap();
    
    if registry.peers.remove(peer_id).is_none() {
        warn!("Attempted to remove unknown peer: {}", peer_id);
        return Ok(());
    }
    
    // Persist to disk
    save_peer_registry()?;
    
    // Notify the peer synchronization system
    sync::peer_removed(peer_id)?;
    
    info!("Peer removed successfully: {}", peer_id);
    Ok(())
}

/// List all known peers
pub fn list_peers() -> Result<Vec<PeerInfo>> {
    let registry = PEER_REGISTRY.lock().unwrap();
    
    let mut peers = Vec::new();
    for (_, peer) in &registry.peers {
        peers.push(PeerInfo {
            id: peer.id.clone(),
            endpoint: peer.endpoint.clone(),
            last_seen: peer.last_seen,
            status: peer.status,
        });
    }
    
    // Sort by ID
    peers.sort_by(|a, b| a.id.cmp(&b.id));
    
    Ok(peers)
}

/// Start synchronizing with a specific peer
pub fn synchronize_with_peer(peer_id: &str) -> Result<()> {
    info!("Starting synchronization with peer: {}", peer_id);
    
    // Check if peer exists
    let registry = PEER_REGISTRY.lock().unwrap();
    
    if !registry.peers.contains_key(peer_id) {
        return Err(anyhow::anyhow!("Unknown peer: {}", peer_id));
    }
    
    let peer = &registry.peers[peer_id];
    
    // Delegate to sync module
    sync::synchronize_with_peer(peer_id, &peer.endpoint)?;
    
    info!("Synchronization started with peer: {}", peer_id);
    Ok(())
}

/// Update peer status
pub fn update_peer_status(peer_id: &str, status: PeerStatus) -> Result<()> {
    let mut registry = PEER_REGISTRY.lock().unwrap();
    
    if let Some(peer) = registry.peers.get_mut(peer_id) {
        peer.status = status;
        peer.last_seen = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        // Persist changes
        save_peer_registry()?;
        
        debug!("Updated status for peer {}: {:?}", peer_id, status);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Unknown peer: {}", peer_id))
    }
}

/// Load peer registry from disk
fn load_peer_registry() -> Result<()> {
    let registry_path = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("peers")
        .join("registry.json");
    
    if !registry_path.exists() {
        debug!("No existing peer registry found, creating new one");
        return Ok(());
    }
    
    // Load the registry
    let registry_json = fs::read_to_string(&registry_path)
        .context("Failed to read peer registry")?;
    
    let loaded_registry: PeerRegistry = serde_json::from_str(&registry_json)
        .context("Failed to parse peer registry JSON")?;
    
    // Update global registry
    let mut registry = PEER_REGISTRY.lock().unwrap();
    *registry = loaded_registry;
    
    debug!("Loaded {} peers from registry", registry.peers.len());
    Ok(())
}

/// Save peer registry to disk
fn save_peer_registry() -> Result<()> {
    let registry_path = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("peers")
        .join("registry.json");
    
    // Ensure parent directory exists
    if let Some(parent) = registry_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Get registry
    let registry = PEER_REGISTRY.lock().unwrap();
    
    // Serialize to JSON
    let registry_json = serde_json::to_string_pretty(&*registry)
        .context("Failed to serialize peer registry")?;
    
    // Write to file
    fs::write(&registry_path, registry_json)
        .context("Failed to write peer registry")?;
    
    debug!("Saved {} peers to registry", registry.peers.len());
    Ok(())
}

/// Peer Registry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerRegistry {
    /// Peers by ID
    peers: HashMap<String, Peer>,
}

impl PeerRegistry {
    /// Create a new peer registry
    fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Peer {
    /// Unique identifier for the peer
    id: String,
    
    /// Network endpoint for the peer
    endpoint: String,
    
    /// Last time the peer was seen (seconds since epoch)
    last_seen: u64,
    
    /// Current peer status
    status: PeerStatus,
    
    /// Synchronization status for different components
    sync_status: HashMap<String, ComponentSyncStatus>,
}

/// Peer information for API responses
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Unique identifier for the peer
    pub id: String,
    
    /// Network endpoint for the peer
    pub endpoint: String,
    
    /// Last time the peer was seen (seconds since epoch)
    pub last_seen: u64,
    
    /// Current peer status
    pub status: PeerStatus,
}

/// Peer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerStatus {
    /// Status is unknown
    Unknown,
    
    /// Peer is online and reachable
    Online,
    
    /// Peer is offline or unreachable
    Offline,
    
    /// Peer is synchronizing
    Synchronizing,
    
    /// Peer is in error state
    Error,
}

/// Component synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComponentSyncStatus {
    /// Component name
    component: String,
    
    /// Last synchronized timestamp
    last_sync: u64,
    
    /// Hash of the last synchronized state
    state_hash: String,
    
    /// Synchronization progress (0-100)
    progress: u8,
}

/// Find peers on the local network
pub fn discover_peers() -> Result<Vec<PeerInfo>> {
    info!("Discovering peers on local network");
    
    // TODO: Implement actual peer discovery using UDP broadcast or similar
    // For now, this is just a placeholder
    
    debug!("Peer discovery not fully implemented yet");
    
    // Return already known peers as a placeholder
    list_peers()
}
