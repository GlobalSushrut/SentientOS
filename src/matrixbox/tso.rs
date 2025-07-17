// SentientOS MatrixBox TSO Format Module
// Handles the Tree-Trie Storage Object (TSO) container format

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use blake3;

use crate::core::constants;
use super::container::{Container, ContainerId};

// TSO file magic number and version
const TSO_MAGIC: [u8; 4] = [b'T', b'S', b'O', b'1'];

/// Create a TSO archive from a container directory
pub fn create_tso_archive(container: &Container, output_path: &Path) -> Result<()> {
    info!("Creating TSO archive for container: {}", container.name);
    
    let container_path = container.path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Container has no path"))?;
    
    // Ensure the container has all required files
    let meta_path = container_path.join("meta.yaml");
    let wasm_path = container_path.join("main.wasm");
    let permissions_path = container_path.join("permissions.zky");
    
    if !meta_path.exists() || !wasm_path.exists() || !permissions_path.exists() {
        return Err(anyhow::anyhow!("Container is missing required files"));
    }
    
    // Create TSO manifest
    let manifest = TsoManifest {
        name: container.name.clone(),
        version: container.version.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        wasm_size: fs::metadata(&wasm_path)?.len(),
        wasm_hash: calculate_file_hash(&wasm_path)?,
        iot_optimized: true,
        files: vec![
            TsoFileEntry {
                name: "meta.yaml".to_string(),
                size: fs::metadata(&meta_path)?.len(),
                offset: 0, // Will be filled in later
                hash: calculate_file_hash(&meta_path)?,
            },
            TsoFileEntry {
                name: "main.wasm".to_string(),
                size: fs::metadata(&wasm_path)?.len(),
                offset: 0, // Will be filled in later
                hash: calculate_file_hash(&wasm_path)?,
            },
            TsoFileEntry {
                name: "permissions.zky".to_string(),
                size: fs::metadata(&permissions_path)?.len(),
                offset: 0, // Will be filled in later
                hash: calculate_file_hash(&permissions_path)?,
            },
        ],
    };
    
    // Create TSO file
    let mut file = File::create(output_path)
        .with_context(|| format!("Failed to create TSO file: {:?}", output_path))?;
    
    // Write TSO header
    file.write_all(&TSO_MAGIC)?;
    
    // Serialize and write the manifest
    let manifest_bytes = bincode::serialize(&manifest)?;
    let manifest_len = manifest_bytes.len() as u32;
    file.write_all(&manifest_len.to_le_bytes())?;
    file.write_all(&manifest_bytes)?;
    
    // Calculate initial offset for file data
    let header_size = TSO_MAGIC.len() + std::mem::size_of::<u32>() + manifest_bytes.len();
    let mut current_offset = header_size;
    
    // Write meta.yaml
    let meta_content = fs::read(&meta_path)?;
    file.write_all(&meta_content)?;
    current_offset += meta_content.len();
    
    // Write main.wasm
    let wasm_content = fs::read(&wasm_path)?;
    file.write_all(&wasm_content)?;
    current_offset += wasm_content.len();
    
    // Write permissions.zky
    let permissions_content = fs::read(&permissions_path)?;
    file.write_all(&permissions_content)?;
    
    info!("Successfully created TSO archive: {:?}", output_path);
    Ok(())
}

/// Extract a TSO archive to a container directory
pub fn extract_tso_archive(archive_path: &Path, target_dir: &Path) -> Result<Container> {
    info!("Extracting TSO archive: {:?}", archive_path);
    
    // Ensure target directory exists
    fs::create_dir_all(target_dir)?;
    
    // Open the TSO file
    let mut file = File::open(archive_path)?;
    
    // Read and verify magic number
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    
    if magic != TSO_MAGIC {
        return Err(anyhow::anyhow!("Invalid TSO file format"));
    }
    
    // Read manifest size
    let mut manifest_size_bytes = [0u8; 4];
    file.read_exact(&mut manifest_size_bytes)?;
    let manifest_size = u32::from_le_bytes(manifest_size_bytes) as usize;
    
    // Read manifest
    let mut manifest_bytes = vec![0u8; manifest_size];
    file.read_exact(&mut manifest_bytes)?;
    
    let manifest: TsoManifest = bincode::deserialize(&manifest_bytes)?;
    
    // Calculate header size
    let header_size = 4 + 4 + manifest_bytes.len();
    let mut current_offset = header_size;
    
    // Extract files
    for file_entry in &manifest.files {
        let target_path = target_dir.join(&file_entry.name);
        
        // Read file content
        let mut content = vec![0u8; file_entry.size as usize];
        file.read_exact(&mut content)?;
        
        // Verify hash
        let hash = blake3::hash(&content);
        if hash.to_hex().to_string() != file_entry.hash {
            return Err(anyhow::anyhow!("Hash verification failed for file: {}", file_entry.name));
        }
        
        // Write file
        fs::write(&target_path, content)?;
        current_offset += file_entry.size as usize;
    }
    
    // Load the extracted container
    let container_path = target_dir.to_str().unwrap();
    let container = super::container::load_container(container_path)?;
    
    info!("Successfully extracted TSO archive: {:?}", archive_path);
    Ok(container)
}

/// Check if a file is a valid TSO archive
pub fn is_valid_tso_archive(path: &Path) -> Result<bool> {
    // Open the file
    let mut file = File::open(path)?;
    
    // Read magic number
    let mut magic = [0u8; 4];
    if file.read_exact(&mut magic).is_err() {
        return Ok(false);
    }
    
    // Verify magic number
    Ok(magic == TSO_MAGIC)
}

/// Get TSO archive info without extracting
pub fn get_tso_info(path: &Path) -> Result<TsoInfo> {
    // Open the file
    let mut file = File::open(path)?;
    
    // Read and verify magic number
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    
    if magic != TSO_MAGIC {
        return Err(anyhow::anyhow!("Invalid TSO file format"));
    }
    
    // Read manifest size
    let mut manifest_size_bytes = [0u8; 4];
    file.read_exact(&mut manifest_size_bytes)?;
    let manifest_size = u32::from_le_bytes(manifest_size_bytes) as usize;
    
    // Read manifest
    let mut manifest_bytes = vec![0u8; manifest_size];
    file.read_exact(&mut manifest_bytes)?;
    
    let manifest: TsoManifest = bincode::deserialize(&manifest_bytes)?;
    
    // Create TSO info
    let info = TsoInfo {
        name: manifest.name,
        version: manifest.version,
        created_at: manifest.created_at,
        wasm_size: manifest.wasm_size,
        iot_optimized: manifest.iot_optimized,
        file_count: manifest.files.len(),
    };
    
    Ok(info)
}

/// Calculate the Blake3 hash of a file
fn calculate_file_hash(path: &Path) -> Result<String> {
    // Open the file
    let mut file = File::open(path)?;
    
    // Read the file content
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    
    // Calculate hash
    let hash = blake3::hash(&content);
    
    Ok(hash.to_hex().to_string())
}

/// TSO manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TsoManifest {
    /// Container name
    name: String,
    
    /// Container version
    version: String,
    
    /// Creation timestamp
    created_at: String,
    
    /// Size of the WASM module
    wasm_size: u64,
    
    /// Hash of the WASM module
    wasm_hash: String,
    
    /// IoT optimization flag
    iot_optimized: bool,
    
    /// Files in the archive
    files: Vec<TsoFileEntry>,
}

/// TSO file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TsoFileEntry {
    /// File name
    name: String,
    
    /// File size
    size: u64,
    
    /// File offset in the archive
    offset: u64,
    
    /// File hash (Blake3)
    hash: String,
}

/// TSO archive info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsoInfo {
    /// Container name
    pub name: String,
    
    /// Container version
    pub version: String,
    
    /// Creation timestamp
    pub created_at: String,
    
    /// Size of the WASM module
    pub wasm_size: u64,
    
    /// IoT optimization flag
    pub iot_optimized: bool,
    
    /// Number of files in the archive
    pub file_count: usize,
}

/// TSO file structure:
/// ```
/// +----------------+
/// | Magic (4 bytes) |
/// +----------------+
/// | Manifest Size  |
/// | (4 bytes)      |
/// +----------------+
/// | Manifest       |
/// | (bincode)      |
/// +----------------+
/// | File 1 Data    |
/// +----------------+
/// | File 2 Data    |
/// +----------------+
/// | ...            |
/// +----------------+
/// ```
