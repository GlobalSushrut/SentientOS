// SentientOS Gossip Verification Module
// Handles cross-device verification of runtime traces

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use serde::{Serialize, Deserialize};
use blake3;

use crate::core::constants;
use super::protocol;
use super::peers;

/// Initialize the trace verification system
pub fn init() -> Result<()> {
    info!("Initializing trace verification system");
    
    let verify_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("verify");
    fs::create_dir_all(&verify_dir)?;
    
    info!("Trace verification system initialized successfully");
    Ok(())
}

/// Shutdown the trace verification system
pub fn shutdown() -> Result<()> {
    info!("Shutting down trace verification system");
    
    // Update cached hashes before shutdown
    let cache_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("hash_cache");
    
    // Ensure cache directory exists
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    
    // Try to refresh peer hashes before shutdown
    let _ = refresh_cached_hashes(&cache_dir);
    
    info!("Trace verification system shutdown complete");
    Ok(())
}

/// Verify trace integrity with peers
pub fn verify_trace() -> Result<VerificationResult> {
    info!("Verifying trace integrity with peers");
    
    // Get local trace hash
    let local_hash = compute_local_trace_hash()?;
    
    // Collect trace hashes from peers
    let peer_hashes = collect_peer_trace_hashes()?;
    
    if peer_hashes.is_empty() {
        info!("No peers available for verification");
        return Ok(VerificationResult {
            verified: true,
            status: VerificationStatus::NoVerification,
            matching_peers: 0,
            total_peers: 0,
            mismatch_details: Vec::new(),
        });
    }
    
    // Compare local hash with peer hashes
    let mut matching_peers = 0;
    let mut mismatch_details = Vec::new();
    
    for (peer_id, hash) in &peer_hashes {
        if hash == &local_hash {
            matching_peers += 1;
        } else {
            mismatch_details.push(TraceMismatch {
                peer_id: peer_id.clone(),
                local_hash: local_hash.clone(),
                peer_hash: hash.clone(),
            });
        }
    }
    
    // Determine verification status
    let status = if matching_peers == peer_hashes.len() {
        VerificationStatus::FullMatch
    } else if matching_peers > 0 {
        VerificationStatus::PartialMatch
    } else {
        VerificationStatus::NoMatch
    };
    
    let verified = matching_peers > 0;
    
    // Record verification result
    record_verification_result(&local_hash, &peer_hashes, &status)?;
    
    let result = VerificationResult {
        verified,
        status,
        matching_peers,
        total_peers: peer_hashes.len(),
        mismatch_details,
    };
    
    info!("Trace verification result: {:?}", result.status);
    Ok(result)
}

/// Compute hash of local trace
fn compute_local_trace_hash() -> Result<String> {
    debug!("Computing local trace hash");
    
    // Get the runtime trace directory
    let runtime_dir = PathBuf::from(constants::ROOT_DIR).join(".runtime");
    
    // Use blake3 to hash directory contents
    let mut hasher = blake3::Hasher::new();
    
    // Hash all trace files in chronological order
    let mut trace_files = Vec::new();
    for entry in fs::read_dir(&runtime_dir)
        .with_context(|| format!("Failed to read runtime directory: {:?}", runtime_dir))?
    {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("trace") {
            trace_files.push(path);
        }
    }
    
    // Sort by filename (which should contain timestamps)
    trace_files.sort();
    
    // Hash all files
    for file_path in &trace_files {
        let content = fs::read(file_path)
            .with_context(|| format!("Failed to read trace file: {:?}", file_path))?;
        hasher.update(&content);
    }
    
    // Get the hash
    let hash = hasher.finalize();
    let hash_hex = hash.to_hex().to_string();
    
    debug!("Local trace hash: {}", hash_hex);
    Ok(hash_hex)
}

/// Collect trace hashes from peers
fn collect_peer_trace_hashes() -> Result<HashMap<String, String>> {
    debug!("Collecting trace hashes from peers");
    
    let peers = super::list_peers()?;
    let mut peer_hashes = HashMap::new();
    
    // Load cached hashes for backup if no peers are available
    let cache_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("hash_cache");
    let cached_hashes = load_cached_peer_hashes(&cache_dir)?;
    let mut used_cached = false;
    
    // Try to collect hashes from online peers
    let mut online_peer_count = 0;
    for peer in &peers {
        if peer.status == super::PeerStatus::Online {
            online_peer_count += 1;
            match protocol::get_trace_hash(&peer.id, &peer.endpoint) {
                Ok(hash) => {
                    // Save hash to cache
                    save_hash_to_cache(&cache_dir, &peer.id, &hash)?;
                    peer_hashes.insert(peer.id.clone(), hash);
                }
                Err(e) => {
                    warn!("Failed to get trace hash from peer {}: {:?}", peer.id, e);
                    // Try to use cached hash as fallback
                    if let Some(cached_hash) = cached_hashes.get(&peer.id) {
                        info!("Using cached hash for peer {}", peer.id);
                        peer_hashes.insert(peer.id.clone(), cached_hash.clone());
                    }
                }
            }
        } else if let Some(cached_hash) = cached_hashes.get(&peer.id) {
            // For offline peers, use cached hash with warning
            warn!("Peer {} is offline, using cached hash", peer.id);
            peer_hashes.insert(peer.id.clone(), cached_hash.clone());
            used_cached = true;
        }
    }
    
    // If no peers available at all, try to use all cached hashes
    if peer_hashes.is_empty() && online_peer_count == 0 && !cached_hashes.is_empty() {
        warn!("No online peers available, using all cached hashes");
        peer_hashes = cached_hashes;
        used_cached = true;
    }
    
    debug!("Collected {} peer trace hashes{}", 
           peer_hashes.len(), 
           if used_cached { " (some from cache)" } else { "" });
    
    Ok(peer_hashes)
}

/// Record verification result for future reference
fn record_verification_result(
    local_hash: &str,
    peer_hashes: &HashMap<String, String>,
    status: &VerificationStatus,
) -> Result<()> {
    let verify_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("verify");
    
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let result_path = verify_dir.join(format!("verify-{}.json", timestamp));
    
    let result = VerificationRecord {
        timestamp,
        local_hash: local_hash.to_string(),
        peer_hashes: peer_hashes.clone(),
        status: *status,
    };
    
    let result_json = serde_json::to_string_pretty(&result)?;
    fs::write(result_path, result_json)?;
    
    Ok(())
}

/// Pull runtime trace from a peer
pub fn pull_from_peer(peer_id: &str) -> Result<()> {
    info!("Pulling runtime trace from peer: {}", peer_id);
    
    // Get peer info
    let peers = super::list_peers()?;
    let peer = peers.iter()
        .find(|p| p.id == peer_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown peer: {}", peer_id))?;
    
    if peer.status != super::PeerStatus::Online {
        return Err(anyhow::anyhow!("Peer is not online: {}", peer_id));
    }
    
    // Get peer's trace hash
    let peer_hash = protocol::get_trace_hash(peer_id, &peer.endpoint)?;
    
    // Get list of trace files from peer
    let trace_files = protocol::list_trace_files(peer_id, &peer.endpoint)?;
    
    // Create directory for pulled trace
    let pull_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("pull")
        .join(peer_id)
        .join(&peer_hash[0..8]); // Use first 8 chars of hash as directory name
    
    fs::create_dir_all(&pull_dir)?;
    
    // Pull each trace file
    for file_info in trace_files {
        info!("Pulling trace file: {}", file_info.name);
        
        let content = protocol::get_trace_file(peer_id, &peer.endpoint, &file_info.name)?;
        
        let file_path = pull_dir.join(&file_info.name);
        fs::write(&file_path, content)?;
    }
    
    // Verify the pulled trace
    let mut hasher = blake3::Hasher::new();
    for entry in fs::read_dir(&pull_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let content = fs::read(&path)?;
            hasher.update(&content);
        }
    }
    
    let hash = hasher.finalize();
    let hash_hex = hash.to_hex().to_string();
    
    if hash_hex != peer_hash {
        return Err(anyhow::anyhow!("Trace verification failed: hash mismatch"));
    }
    
    // Create verification record
    let record = PullRecord {
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        peer_id: peer_id.to_string(),
        hash: peer_hash,
        files_count: trace_files.len(),
        verified: true,
    };
    
    let record_path = pull_dir.join("pull-record.json");
    fs::write(&record_path, serde_json::to_string_pretty(&record)?)?;
    
    info!("Successfully pulled trace from peer: {}", peer_id);
    Ok(())
}

/// Enable trace sync with peers
pub fn enable_sync() -> Result<()> {
    info!("Enabling trace synchronization with peers");
    
    // Create sync config file
    let config_path = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("sync")
        .join("config.json");
    
    // Create hash cache directory
    let cache_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("hash_cache");
    fs::create_dir_all(&cache_dir)?;
    
    let config = SyncConfig {
        enabled: true,
        auto_verify: true,
        pull_interval_seconds: 3600, // Default: sync once per hour
        verification_interval_seconds: 1800, // Default: verify every 30 minutes
        use_cached_hashes: true,      // Use cached hashes when peers are unavailable
        max_cache_age_hours: 24,      // Cache valid for 24 hours
    };
    
    fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
    
    info!("Trace synchronization enabled");
    Ok(())
}

/// Verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// No verification performed (no peers available)
    NoVerification,
    
    /// All peers match local trace
    FullMatch,
    
    /// Some peers match local trace
    PartialMatch,
    
    /// No peers match local trace
    NoMatch,
}

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the trace is verified (at least one peer matches)
    pub verified: bool,
    
    /// Verification status
    pub status: VerificationStatus,
    
    /// Number of peers with matching trace
    pub matching_peers: usize,
    
    /// Total number of peers checked
    pub total_peers: usize,
    
    /// Details of mismatches
    pub mismatch_details: Vec<TraceMismatch>,
}

/// Trace mismatch details
#[derive(Debug, Clone)]
pub struct TraceMismatch {
    /// Peer ID
    pub peer_id: String,
    
    /// Local trace hash
    pub local_hash: String,
    
    /// Peer trace hash
    pub peer_hash: String,
}

/// Verification record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerificationRecord {
    /// Timestamp of verification
    timestamp: u64,
    
    /// Local trace hash
    local_hash: String,
    
    /// Peer trace hashes
    peer_hashes: HashMap<String, String>,
    
    /// Verification status
    status: VerificationStatus,
}

/// Trace file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceFileInfo {
    /// File name
    pub name: String,
    
    /// File size in bytes
    pub size: u64,
    
    /// File hash
    pub hash: String,
}

/// Pull record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PullRecord {
    /// Timestamp of pull
    timestamp: u64,
    
    /// Peer ID
    peer_id: String,
    
    /// Trace hash
    hash: String,
    
    /// Number of files pulled
    files_count: usize,
    
    /// Whether the trace was verified
    verified: bool,
}

/// Cached peer hash record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedHashRecord {
    /// Peer ID
    peer_id: String,
    
    /// Trace hash
    hash: String,
    
    /// Timestamp when hash was cached
    timestamp: u64,
    
    /// Source of the hash (direct or inferred)
    source: CachedHashSource,
}

/// Source of a cached hash
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CachedHashSource {
    /// Directly retrieved from peer
    Direct,
    
    /// Inferred from other peers (consensus)
    Inferred,
    
    /// Manually approved by user
    Approved,
}

/// Save a hash to the cache
fn save_hash_to_cache(cache_dir: &Path, peer_id: &str, hash: &str) -> Result<()> {
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?;
    }
    
    let record = CachedHashRecord {
        peer_id: peer_id.to_string(),
        hash: hash.to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        source: CachedHashSource::Direct,
    };
    
    let file_path = cache_dir.join(format!("{}.json", peer_id));
    fs::write(&file_path, serde_json::to_string_pretty(&record)?)?;
    
    Ok(())
}

/// Load cached peer hashes
fn load_cached_peer_hashes(cache_dir: &Path) -> Result<HashMap<String, String>> {
    let mut hashes = HashMap::new();
    
    // Create directory if it doesn't exist
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?;
        return Ok(hashes);
    }
    
    // Read config to get max cache age
    let config_path = PathBuf::from(constants::ROOT_DIR)
        .join(".gossip")
        .join("sync")
        .join("config.json");
    
    let max_age_hours = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<SyncConfig>(&content) {
                    Ok(config) => config.max_cache_age_hours,
                    Err(_) => 24, // Default: 24 hours
                }
            }
            Err(_) => 24, // Default: 24 hours
        }
    } else {
        24 // Default: 24 hours
    };
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let max_age_secs = max_age_hours * 3600;
    
    for entry in fs::read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<CachedHashRecord>(&content) {
                        Ok(record) => {
                            // Check if record is still valid (not too old)
                            if now - record.timestamp <= max_age_secs {
                                hashes.insert(record.peer_id, record.hash);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse cached hash record {:?}: {:?}", path, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read cached hash file {:?}: {:?}", path, e);
                }
            }
        }
    }
    
    Ok(hashes)
}

/// Refresh cached hashes based on current peer state
fn refresh_cached_hashes(cache_dir: &Path) -> Result<()> {
    debug!("Refreshing cached peer hashes");
    
    let peers = super::list_peers()?;
    let mut refreshed_count = 0;
    
    for peer in &peers {
        if peer.status == super::PeerStatus::Online {
            match protocol::get_trace_hash(&peer.id, &peer.endpoint) {
                Ok(hash) => {
                    save_hash_to_cache(cache_dir, &peer.id, &hash)?;
                    refreshed_count += 1;
                }
                Err(e) => {
                    debug!("Could not refresh hash for peer {}: {:?}", peer.id, e);
                }
            }
        }
    }
    
    debug!("Refreshed {} peer hashes in cache", refreshed_count);
    Ok(())
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncConfig {
    /// Whether sync is enabled
    enabled: bool,
    
    /// Whether to automatically verify after sync
    auto_verify: bool,
    
    /// How often to pull from peers (seconds)
    pull_interval_seconds: u64,
    
    /// How often to verify (seconds)
    verification_interval_seconds: u64,
    
    /// Whether to use cached hashes when peers are unavailable
    use_cached_hashes: bool,
    
    /// Maximum age of cached hashes in hours
    max_cache_age_hours: u64,
}
