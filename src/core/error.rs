use thiserror::Error;

/// Core SentientOS errors
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("File system error: {0}")]
    FileSystem(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("ZK verification failed: {0}")]
    ZkVerificationFailed(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("Container error: {0}")]
    Container(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("System panic: {0}")]
    Panic(String),
    
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
}
