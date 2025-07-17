use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::PathBuf;
use std::fs;
use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Serialize, Deserialize};
use blake3;

use crate::core::constants;

const PROTOCOL_VERSION: u8 = 1;
const MAX_MESSAGE_SIZE: usize = 65507; // Max UDP packet size
const DEFAULT_PORT: u16 = 29876;
const DISCOVERY_PORT: u16 = 29877;
const HEARTBEAT_INTERVAL: u64 = 30; // seconds

// Global protocol state
lazy_static::lazy_static! {
    static ref PROTOCOL_STATE: Arc<Mutex<ProtocolState>> = 
        Arc::new(Mutex::new(ProtocolState::new()));
}

/// Initialize the gossip protocol subsystem
pub fn init() -> Result<()> {
    info!("Initializing gossip protocol subsystem");
    
    // Create protocol directories
    let protocol_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("protocol");
    
    fs::create_dir_all(&protocol_dir)?;
    
    // Initialize the protocol state
    let mut state = PROTOCOL_STATE.lock().unwrap();
    *state = load_protocol_state()?;
    
    // Start the background listener thread if enabled
    if state.enabled {
        start_listener_thread()?;
    }
    
    info!("Gossip protocol subsystem initialized");
    Ok(())
}

/// Shutdown the gossip protocol subsystem
pub fn shutdown() -> Result<()> {
    info!("Shutting down gossip protocol subsystem");
    
    let mut state = PROTOCOL_STATE.lock().unwrap();
    
    // Save current state
    save_protocol_state(&*state)?;
    
    // Signal threads to stop
    state.enabled = false;
    
    info!("Gossip protocol subsystem shutdown complete");
    Ok(())
}

/// Enable the gossip protocol
pub fn enable() -> Result<()> {
    let mut state = PROTOCOL_STATE.lock().unwrap();
    
    if !state.enabled {
        state.enabled = true;
        start_listener_thread()?;
        info!("Gossip protocol enabled");
    }
    
    save_protocol_state(&*state)?;
    Ok(())
}

/// Disable the gossip protocol
pub fn disable() -> Result<()> {
    let mut state = PROTOCOL_STATE.lock().unwrap();
    
    if state.enabled {
        state.enabled = false;
        info!("Gossip protocol disabled");
    }
    
    save_protocol_state(&*state)?;
    Ok(())
}

/// Set the node identifier
pub fn set_node_id(node_id: &str) -> Result<()> {
    let mut state = PROTOCOL_STATE.lock().unwrap();
    state.node_id = node_id.to_string();
    
    save_protocol_state(&*state)?;
    info!("Node ID set to: {}", node_id);
    Ok(())
}

/// Send a gossip message to a specific peer
pub fn send_message(peer_endpoint: &str, message_type: MessageType, payload: &[u8]) -> Result<()> {
    let state = PROTOCOL_STATE.lock().unwrap();
    
    if !state.enabled {
        return Err(anyhow::anyhow!("Gossip protocol is disabled"));
    }
    
    // Parse peer endpoint
    let peer_addr: SocketAddr = peer_endpoint.parse()
        .with_context(|| format!("Invalid peer endpoint: {}", peer_endpoint))?;
    
    // Create message
    let message = Message {
        version: PROTOCOL_VERSION,
        source_id: state.node_id.clone(),
        message_type,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        payload: payload.to_vec(),
        signature: String::new(), // TODO: Implement proper signatures
    };
    
    // Serialize message
    let message_bytes = bincode::serialize(&message)
        .context("Failed to serialize gossip message")?;
    
    // Check message size
    if message_bytes.len() > MAX_MESSAGE_SIZE {
        return Err(anyhow::anyhow!("Message too large: {} bytes (max: {})", 
                                 message_bytes.len(), MAX_MESSAGE_SIZE));
    }
    
    // Send message
    let socket = UdpSocket::bind("0.0.0.0:0")
        .context("Failed to create UDP socket for sending")?;
    
    socket.send_to(&message_bytes, peer_addr)
        .with_context(|| format!("Failed to send gossip message to {}", peer_endpoint))?;
    
    debug!("Sent gossip message to {}: {:?}", peer_endpoint, message.message_type);
    Ok(())
}

/// Send a discovery ping to find peers
pub fn send_discovery_ping() -> Result<()> {
    // Create discovery message
    let state = PROTOCOL_STATE.lock().unwrap();
    
    if !state.enabled {
        return Err(anyhow::anyhow!("Gossip protocol is disabled"));
    }
    
    // Create discovery payload with node information
    let discovery_info = DiscoveryInfo {
        node_id: state.node_id.clone(),
        capabilities: state.capabilities.clone(),
        version: state.version.clone(),
    };
    
    let payload = bincode::serialize(&discovery_info)
        .context("Failed to serialize discovery info")?;
    
    // Broadcast to discovery address
    let socket = UdpSocket::bind("0.0.0.0:0")
        .context("Failed to create UDP socket for discovery")?;
    
    socket.set_broadcast(true)
        .context("Failed to set broadcast option")?;
    
    let broadcast_addr = format!("255.255.255.255:{}", DISCOVERY_PORT);
    
    socket.send_to(&payload, &broadcast_addr)
        .context("Failed to send discovery ping")?;
    
    debug!("Sent discovery ping");
    Ok(())
}

/// Start the background listener thread
fn start_listener_thread() -> Result<()> {
    let state_arc = Arc::clone(&PROTOCOL_STATE);
    
    thread::spawn(move || {
        if let Err(e) = run_listener_loop(state_arc) {
            error!("Gossip listener thread error: {}", e);
        }
    });
    
    debug!("Started gossip listener thread");
    Ok(())
}

/// Main listener loop
fn run_listener_loop(state_arc: Arc<Mutex<ProtocolState>>) -> Result<()> {
    let addr = format!("0.0.0.0:{}", DEFAULT_PORT);
    let socket = UdpSocket::bind(&addr)
        .with_context(|| format!("Failed to bind to {}", addr))?;
    
    let discovery_addr = format!("0.0.0.0:{}", DISCOVERY_PORT);
    let discovery_socket = UdpSocket::bind(&discovery_addr)
        .with_context(|| format!("Failed to bind to {}", discovery_addr))?;
    
    info!("Gossip listener active on {} and {}", addr, discovery_addr);
    
    let mut buffer = [0u8; MAX_MESSAGE_SIZE];
    
    // Set socket to non-blocking mode
    socket.set_nonblocking(true)?;
    discovery_socket.set_nonblocking(true)?;
    
    // Run until disabled
    loop {
        // Check if protocol is still enabled
        if !state_arc.lock().unwrap().enabled {
            break;
        }
        
        // Try to receive regular messages
        match socket.recv_from(&mut buffer) {
            Ok((size, src)) => {
                let message_data = &buffer[..size];
                if let Err(e) = handle_message(message_data, src) {
                    warn!("Error handling gossip message: {}", e);
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No message, continue
            },
            Err(e) => {
                error!("Error receiving gossip message: {}", e);
            }
        }
        
        // Try to receive discovery messages
        match discovery_socket.recv_from(&mut buffer) {
            Ok((size, src)) => {
                let message_data = &buffer[..size];
                if let Err(e) = handle_discovery(message_data, src) {
                    warn!("Error handling discovery message: {}", e);
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No message, continue
            },
            Err(e) => {
                error!("Error receiving discovery message: {}", e);
            }
        }
        
        // Sleep to avoid busy-waiting
        thread::sleep(Duration::from_millis(100));
    }
    
    info!("Gossip listener thread terminated");
    Ok(())
}

/// Handle an incoming gossip message
fn handle_message(message_data: &[u8], src: SocketAddr) -> Result<()> {
    // Deserialize message
    let message: Message = bincode::deserialize(message_data)
        .context("Failed to deserialize gossip message")?;
    
    // Verify protocol version
    if message.version != PROTOCOL_VERSION {
        warn!("Received message with unsupported protocol version: {}", message.version);
        return Ok(());
    }
    
    // Process message based on type
    match message.message_type {
        MessageType::Heartbeat => {
            debug!("Received heartbeat from {}", message.source_id);
            // Update peer last seen time
            super::update_peer_status(&message.source_id, super::PeerStatus::Online)?;
        },
        MessageType::SyncRequest => {
            debug!("Received sync request from {}", message.source_id);
            // Pass to sync module
            super::sync::handle_sync_request(&message.source_id, &message.payload)?;
        },
        MessageType::SyncResponse => {
            debug!("Received sync response from {}", message.source_id);
            // Pass to sync module
            super::sync::handle_sync_response(&message.source_id, &message.payload)?;
        },
        MessageType::StateUpdate => {
            debug!("Received state update from {}", message.source_id);
            // Pass to sync module
            super::sync::handle_state_update(&message.source_id, &message.payload)?;
        },
    }
    
    Ok(())
}

/// Handle a discovery message
fn handle_discovery(message_data: &[u8], src: SocketAddr) -> Result<()> {
    // Deserialize discovery info
    let discovery_info: DiscoveryInfo = bincode::deserialize(message_data)
        .context("Failed to deserialize discovery message")?;
    
    debug!("Received discovery from node: {}", discovery_info.node_id);
    
    // Don't respond to own discovery messages
    let state = PROTOCOL_STATE.lock().unwrap();
    if discovery_info.node_id == state.node_id {
        return Ok(());
    }
    
    // Add peer to registry if not already known
    let endpoint = format!("{}:{}", src.ip(), DEFAULT_PORT);
    drop(state); // Release lock before calling add_peer
    
    // Check if we already know this peer
    let peers = super::list_peers()?;
    let known = peers.iter().any(|p| p.id == discovery_info.node_id);
    
    if !known {
        // Add new peer
        super::add_peer(&discovery_info.node_id, &endpoint)?;
        info!("Discovered new peer: {}", discovery_info.node_id);
    } else {
        // Update existing peer status
        super::update_peer_status(&discovery_info.node_id, super::PeerStatus::Online)?;
        debug!("Updated existing peer from discovery: {}", discovery_info.node_id);
    }
    
    Ok(())
}

/// Load protocol state from disk
fn load_protocol_state() -> Result<ProtocolState> {
    let state_path = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("protocol")
        .join("state.json");
    
    if !state_path.exists() {
        debug!("No existing protocol state found, creating default");
        return Ok(ProtocolState::new());
    }
    
    // Load the state
    let state_json = fs::read_to_string(&state_path)
        .context("Failed to read protocol state")?;
    
    let state: ProtocolState = serde_json::from_str(&state_json)
        .context("Failed to parse protocol state JSON")?;
    
    Ok(state)
}

/// Save protocol state to disk
fn save_protocol_state(state: &ProtocolState) -> Result<()> {
    let state_path = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("protocol")
        .join("state.json");
    
    // Ensure parent directory exists
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Serialize to JSON
    let state_json = serde_json::to_string_pretty(&state)
        .context("Failed to serialize protocol state")?;
    
    // Write to file
    fs::write(&state_path, state_json)
        .context("Failed to write protocol state")?;
    
    Ok(())
}

/// Protocol state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProtocolState {
    /// Node identifier
    node_id: String,
    
    /// Protocol enabled
    enabled: bool,
    
    /// Node capabilities
    capabilities: Vec<String>,
    
    /// Software version
    version: String,
    
    /// Last heartbeat timestamp
    last_heartbeat: u64,
}

impl ProtocolState {
    /// Create new default protocol state
    fn new() -> Self {
        Self {
            node_id: generate_node_id(),
            enabled: true,
            capabilities: vec![
                "sync".to_string(),
                "discovery".to_string(),
            ],
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_heartbeat: 0,
        }
    }
}

/// Generate a unique node ID
fn generate_node_id() -> String {
    use rand::{thread_rng, Rng};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();
    
    let mut rng = thread_rng();
    let random_bytes: [u8; 8] = rng.gen();
    
    // Hash timestamp and random bytes for uniqueness
    let mut hasher = blake3::Hasher::new();
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(&random_bytes);
    
    let hash = hasher.finalize();
    let node_id = hash.to_hex().to_string();
    
    // Use first 16 chars of the hash
    node_id[..16].to_string()
}

/// Get trace hash from a peer
pub fn get_trace_hash(peer_id: &str, peer_endpoint: &str) -> Result<String> {
    debug!("Getting trace hash from peer: {}", peer_id);
    
    // Create request message
    let request_msg = TraceHashRequestMsg {
        request_id: generate_request_id(),
    };
    
    // Serialize request
    let payload = serde_json::to_vec(&request_msg)?;
    
    // Send request
    send_message(peer_endpoint, MessageType::TraceHashRequest, &payload)?;
    
    // TODO: In a real implementation, we would wait for the response asynchronously
    // For now, we'll simulate a response with a dummy hash
    
    // Compute a deterministic hash for the simulation
    let mut hasher = blake3::Hasher::new();
    hasher.update(peer_id.as_bytes());
    hasher.update(b"trace-hash-simulation");
    let hash = hasher.finalize();
    
    Ok(hash.to_hex().to_string())
}

/// List trace files from a peer
pub fn list_trace_files(peer_id: &str, peer_endpoint: &str) -> Result<Vec<super::verify::TraceFileInfo>> {
    debug!("Listing trace files from peer: {}", peer_id);
    
    // Create request message
    let request_msg = ListTraceFilesRequestMsg {
        request_id: generate_request_id(),
    };
    
    // Serialize request
    let payload = serde_json::to_vec(&request_msg)?;
    
    // Send request
    send_message(peer_endpoint, MessageType::ListTraceFilesRequest, &payload)?;
    
    // TODO: In a real implementation, we would wait for the response asynchronously
    // For now, we'll simulate a response with dummy files
    
    let file_count = 3; // Simulate 3 trace files
    let mut files = Vec::with_capacity(file_count);
    
    for i in 0..file_count {
        let filename = format!("trace-{}.trace", i);
        
        // Create deterministic hash for the file
        let mut hasher = blake3::Hasher::new();
        hasher.update(peer_id.as_bytes());
        hasher.update(filename.as_bytes());
        let hash = hasher.finalize();
        
        files.push(super::verify::TraceFileInfo {
            name: filename,
            size: 1024 * (i + 1) as u64, // Simulate different file sizes
            hash: hash.to_hex().to_string(),
        });
    }
    
    Ok(files)
}

/// Get a trace file from a peer
pub fn get_trace_file(peer_id: &str, peer_endpoint: &str, filename: &str) -> Result<Vec<u8>> {
    debug!("Getting trace file from peer: {}, file: {}", peer_id, filename);
    
    // Create request message
    let request_msg = GetTraceFileRequestMsg {
        request_id: generate_request_id(),
        filename: filename.to_string(),
    };
    
    // Serialize request
    let payload = serde_json::to_vec(&request_msg)?;
    
    // Send request
    send_message(peer_endpoint, MessageType::GetTraceFileRequest, &payload)?;
    
    // TODO: In a real implementation, we would wait for the response asynchronously
    // For now, we'll simulate a response with dummy file content
    
    // Create deterministic content for the simulation
    let mut content = Vec::new();
    let content_size = 1024; // 1KB simulated content
    
    // Fill with deterministic pattern based on filename and peer_id
    for i in 0..content_size {
        let byte = (i as u8) ^ (peer_id.as_bytes()[i % peer_id.len()]);
        content.push(byte);
    }
    
    Ok(content)
}

/// Generate a unique request ID
fn generate_request_id() -> String {
    use rand::{thread_rng, Rng};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis();
    
    let mut rng = thread_rng();
    let random_value: u64 = rng.gen();
    
    format!("{:x}-{:x}", timestamp, random_value)
}

/// Message structure for gossip protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    /// Protocol version
    version: u8,
    
    /// Source node ID
    source_id: String,
    
    /// Message type
    message_type: MessageType,
    
    /// Timestamp (seconds since epoch)
    timestamp: u64,
    
    /// Message payload
    payload: Vec<u8>,
    
    /// Message signature
    signature: String,
}

/// Message types for gossip protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Heartbeat to indicate node is alive
    Heartbeat,
    
    /// Request to sync state
    SyncRequest,
    
    /// Response to sync request
    SyncResponse,
    
    /// State update notification
    StateUpdate,
    
    /// Request trace hash
    TraceHashRequest,
    
    /// Trace hash response
    TraceHashResponse,
    
    /// List trace files request
    ListTraceFilesRequest,
    
    /// List trace files response
    ListTraceFilesResponse,
    
    /// Get trace file request
    GetTraceFileRequest,
    
    /// Get trace file response
    GetTraceFileResponse,
}

/// Discovery information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryInfo {
    /// Node identifier
    node_id: String,
    
    /// Node capabilities
    capabilities: Vec<String>,
    
    /// Software version
    version: String,
}

/// Trace hash request message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TraceHashRequestMsg {
    /// Request identifier
    request_id: String,
}

/// Trace hash response message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TraceHashResponseMsg {
    /// Request identifier (matches the request)
    request_id: String,
    
    /// Trace hash
    hash: String,
}

/// List trace files request message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListTraceFilesRequestMsg {
    /// Request identifier
    request_id: String,
}

/// List trace files response message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListTraceFilesResponseMsg {
    /// Request identifier (matches the request)
    request_id: String,
    
    /// List of trace files
    files: Vec<TraceFile>,
}

/// Trace file information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TraceFile {
    /// File name
    name: String,
    
    /// File size in bytes
    size: u64,
    
    /// File hash
    hash: String,
}

/// Get trace file request message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetTraceFileRequestMsg {
    /// Request identifier
    request_id: String,
    
    /// File name to retrieve
    filename: String,
}

/// Get trace file response message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetTraceFileResponseMsg {
    /// Request identifier (matches the request)
    request_id: String,
    
    /// File name
    filename: String,
    
    /// File content (base64 encoded)
    content: String,
}
