// SentientOS Intent System
// Provides developer intent logging & replay

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::core::constants;

// Whether recording is active
static RECORDING_ACTIVE: AtomicBool = AtomicBool::new(false);

// Current session ID
static CURRENT_SESSION: Mutex<Option<String>> = Mutex::new(None);

/// Initialize the intent system
pub fn init() -> Result<()> {
    info!("Initializing SentientOS intent system");
    
    // Create intent system directories
    let intent_dir = PathBuf::from(constants::ROOT_DIR).join(".intent");
    fs::create_dir_all(&intent_dir)?;
    
    let sessions_dir = intent_dir.join("sessions");
    fs::create_dir_all(&sessions_dir)?;
    
    let replay_dir = intent_dir.join("replay");
    fs::create_dir_all(&replay_dir)?;
    
    let timeline_dir = intent_dir.join("timeline");
    fs::create_dir_all(&timeline_dir)?;
    
    info!("SentientOS intent system initialized successfully");
    Ok(())
}

/// Shutdown the intent system
pub fn shutdown() -> Result<()> {
    info!("Shutting down SentientOS intent system");
    
    // Stop recording if active
    if RECORDING_ACTIVE.load(Ordering::SeqCst) {
        stop_recording()?;
    }
    
    info!("SentientOS intent system shutdown complete");
    Ok(())
}

/// Start recording developer intent session
pub fn start_recording() -> Result<String> {
    if RECORDING_ACTIVE.load(Ordering::SeqCst) {
        anyhow::bail!("Recording already in progress");
    }
    
    info!("Starting developer intent recording session");
    
    // Generate session ID
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let session_id = format!("session-{}", timestamp);
    
    // Create session directory
    let session_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".intent")
        .join("sessions")
        .join(&session_id);
    fs::create_dir_all(&session_dir)?;
    
    // Create session metadata
    let now: DateTime<Utc> = SystemTime::now().into();
    let metadata = SessionMetadata {
        id: session_id.clone(),
        started_at: now.to_rfc3339(),
        completed_at: None,
        events_count: 0,
    };
    
    // Write metadata
    let metadata_path = session_dir.join("metadata.json");
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;
    
    // Set current session
    *CURRENT_SESSION.lock().unwrap() = Some(session_id.clone());
    
    // Mark recording as active
    RECORDING_ACTIVE.store(true, Ordering::SeqCst);
    
    info!("Started recording session: {}", session_id);
    Ok(session_id)
}

/// Stop recording developer intent session
pub fn stop_recording() -> Result<()> {
    if !RECORDING_ACTIVE.load(Ordering::SeqCst) {
        anyhow::bail!("No recording in progress");
    }
    
    // Get current session ID
    let session_id = match &*CURRENT_SESSION.lock().unwrap() {
        Some(id) => id.clone(),
        None => anyhow::bail!("No current session found"),
    };
    
    info!("Stopping developer intent recording session: {}", session_id);
    
    // Update session metadata
    let session_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".intent")
        .join("sessions")
        .join(&session_id);
    
    let metadata_path = session_dir.join("metadata.json");
    let metadata_str = fs::read_to_string(&metadata_path)?;
    let mut metadata: SessionMetadata = serde_json::from_str(&metadata_str)?;
    
    let now: DateTime<Utc> = SystemTime::now().into();
    metadata.completed_at = Some(now.to_rfc3339());
    
    // Write updated metadata
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;
    
    // Clear current session
    *CURRENT_SESSION.lock().unwrap() = None;
    
    // Mark recording as inactive
    RECORDING_ACTIVE.store(false, Ordering::SeqCst);
    
    info!("Stopped recording session: {}", session_id);
    Ok(())
}

/// Record an intent event
pub fn record_event(event_type: &str, details: &str) -> Result<()> {
    if !RECORDING_ACTIVE.load(Ordering::SeqCst) {
        // No recording in progress, just ignore
        return Ok(());
    }
    
    // Get current session ID
    let session_id = match &*CURRENT_SESSION.lock().unwrap() {
        Some(id) => id.clone(),
        None => return Ok(()), // No current session, ignore
    };
    
    debug!("Recording intent event: {}", event_type);
    
    // Get event timestamp
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    
    // Create event record
    let event = IntentEvent {
        timestamp,
        event_type: event_type.to_string(),
        details: details.to_string(),
    };
    
    // Write event to session directory
    let session_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".intent")
        .join("sessions")
        .join(&session_id);
    
    let event_path = session_dir.join(format!("event-{}.json", timestamp));
    fs::write(&event_path, serde_json::to_string_pretty(&event)?)?;
    
    // Update metadata event count
    let metadata_path = session_dir.join("metadata.json");
    let metadata_str = fs::read_to_string(&metadata_path)?;
    let mut metadata: SessionMetadata = serde_json::from_str(&metadata_str)?;
    metadata.events_count += 1;
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;
    
    Ok(())
}

/// Replay a recorded session
pub fn replay_session(session_id: &str) -> Result<()> {
    info!("Replaying intent session: {}", session_id);
    
    // Get session directory
    let session_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".intent")
        .join("sessions")
        .join(session_id);
    
    if !session_dir.exists() {
        anyhow::bail!("Session not found: {}", session_id);
    }
    
    // Read session metadata
    let metadata_path = session_dir.join("metadata.json");
    let metadata_str = fs::read_to_string(&metadata_path)?;
    let metadata: SessionMetadata = serde_json::from_str(&metadata_str)?;
    
    info!("Replaying session: {} (events: {})", session_id, metadata.events_count);
    
    // Collect all events
    let mut events = Vec::new();
    for entry in fs::read_dir(&session_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.starts_with("event-")) {
            let content = fs::read_to_string(&path)?;
            let event: IntentEvent = serde_json::from_str(&content)?;
            events.push(event);
        }
    }
    
    // Sort events by timestamp
    events.sort_by_key(|e| e.timestamp);
    
    // Replay events
    for event in events {
        info!("[REPLAY] {}: {}", event.event_type, event.details);
        
        // In a real implementation, we would actually execute the intent
        // For now, we just log it
    }
    
    info!("Completed replaying session: {}", session_id);
    Ok(())
}

/// Session metadata
#[derive(Debug, Serialize, Deserialize)]
struct SessionMetadata {
    /// Session ID
    id: String,
    
    /// When the session was started
    started_at: String,
    
    /// When the session was completed
    completed_at: Option<String>,
    
    /// Number of events in the session
    events_count: usize,
}

/// Intent event
#[derive(Debug, Serialize, Deserialize)]
struct IntentEvent {
    /// Event timestamp
    timestamp: u64,
    
    /// Event type
    event_type: String,
    
    /// Event details
    details: String,
}

/// List all recorded sessions
pub fn list_sessions() -> Result<Vec<SessionMetadata>> {
    info!("Listing intent sessions");
    
    let sessions_dir = PathBuf::from(constants::ROOT_DIR)
        .join(".intent")
        .join("sessions");
    
    let mut sessions = Vec::new();
    for entry in fs::read_dir(sessions_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            let metadata_path = entry.path().join("metadata.json");
            if metadata_path.exists() {
                let metadata_str = fs::read_to_string(&metadata_path)?;
                let metadata: SessionMetadata = serde_json::from_str(&metadata_str)?;
                sessions.push(metadata);
            }
        }
    }
    
    info!("Found {} intent sessions", sessions.len());
    Ok(sessions)
}
