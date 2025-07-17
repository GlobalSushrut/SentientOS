// SentientOS ZK Module
// Handles zero-knowledge proofs and ZK-YAML contract verification

pub mod contracts;
pub mod verify;
pub mod parser;
pub mod executor;

use anyhow::Result;
use tracing::{info, warn};
use std::path::PathBuf;

/// Initialize the ZK subsystem
pub fn init() -> Result<()> {
    info!("Initializing ZK subsystem");
    
    // Create necessary ZK directories if they don't exist
    crate::core::fs::create_directory_if_not_exists(".zk")?;
    crate::core::fs::create_directory_if_not_exists(".zk/contracts")?;
    crate::core::fs::create_directory_if_not_exists(".zk/proofs")?;
    crate::core::fs::create_directory_if_not_exists(".zk/keys")?;
    crate::core::fs::create_directory_if_not_exists(".zk/runtime")?;
    
    // Initialize ZK verification system
    verify::init()?;
    
    // Initialize ZK-YAML parser
    parser::init()?;
    
    // Initialize ZK contract executor
    executor::init()?;
    
    info!("ZK subsystem initialized successfully");
    Ok(())
}

/// Shutdown the ZK subsystem
pub fn shutdown() -> Result<()> {
    info!("Shutting down ZK subsystem");
    
    // Shutdown components in reverse order
    executor::shutdown()?;
    parser::shutdown()?;
    verify::shutdown()?;
    
    info!("ZK subsystem shutdown complete");
    Ok(())
}

/// Load and parse a ZK-YAML contract
pub fn load_contract(path: &str) -> Result<contracts::ZkContract> {
    let full_path = PathBuf::from(crate::core::constants::ROOT_DIR).join(path);
    info!("Loading ZK contract from: {:?}", full_path);
    
    // Read the contract file
    let contract_content = std::fs::read_to_string(&full_path)?;
    
    // Parse the ZK-YAML contract
    let contract = parser::parse_zk_yaml(&contract_content)?;
    
    info!("Successfully loaded ZK contract: {}", contract.name);
    Ok(contract)
}

/// Verify a ZK contract's integrity
pub fn verify_contract(contract: &contracts::ZkContract) -> Result<bool> {
    info!("Verifying ZK contract: {}", contract.name);
    
    // Use the verify module to check the contract's integrity
    let result = verify::verify_contract(contract)?;
    
    if result {
        info!("ZK contract verification successful: {}", contract.name);
    } else {
        warn!("ZK contract verification failed: {}", contract.name);
    }
    
    Ok(result)
}

/// Generate a ZK proof for a given operation
pub fn generate_proof(data: &[u8], operation: &str) -> Result<Vec<u8>> {
    info!("Generating ZK proof for operation: {}", operation);
    
    // Use the verify module to generate a proof
    let proof = verify::generate_proof(data, operation)?;
    
    info!("Successfully generated ZK proof for operation: {}", operation);
    Ok(proof)
}

/// Verify a ZK proof for a given operation
pub fn verify_proof(data: &[u8], proof: &[u8], operation: &str) -> Result<bool> {
    info!("Verifying ZK proof for operation: {}", operation);
    
    // Use the verify module to verify the proof
    let result = verify::verify_proof(data, proof, operation)?;
    
    if result {
        info!("ZK proof verification successful for operation: {}", operation);
    } else {
        warn!("ZK proof verification failed for operation: {}", operation);
    }
    
    Ok(result)
}

/// Execute a ZK contract method
pub fn execute_contract_method(
    contract: &contracts::ZkContract,
    method_name: &str,
    args: &[serde_json::Value],
) -> Result<serde_json::Value> {
    info!("Executing ZK contract method: {}.{}", contract.name, method_name);
    
    // Verify contract first
    let verified = verify_contract(contract)?;
    if !verified {
        return Err(anyhow::anyhow!("Cannot execute unverified contract: {}", contract.name));
    }
    
    // Execute the method using the executor
    let result = executor::execute_contract_method(contract, method_name, args)?;
    
    info!("Successfully executed ZK contract method: {}.{}", contract.name, method_name);
    Ok(result)
}
