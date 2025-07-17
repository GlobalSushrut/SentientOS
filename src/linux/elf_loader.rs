// SentientOS ELF Binary Loader
// Provides ELF binary loading and execution capabilities

use anyhow::{Result, Context, anyhow};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::collections::HashMap;
use goblin::elf::{Elf, ProgramHeader, SectionHeader, header};
use goblin::Object;
use scroll::Pread;

use crate::core::constants;
use super::compatibility;

/// ELF file header and information
#[derive(Debug)]
pub struct ElfInfo {
    /// Path to the ELF binary
    pub path: PathBuf,
    
    /// Architecture
    pub arch: ElfArchitecture,
    
    /// Entry point address
    pub entry_point: u64,
    
    /// Required shared libraries
    pub shared_libs: Vec<String>,
    
    /// Program headers
    pub program_headers: Vec<ElfProgramHeader>,
    
    /// Section headers
    pub section_headers: Vec<ElfSectionHeader>,
    
    /// Is executable (vs. shared library)
    pub is_executable: bool,
    
    /// Is dynamically linked
    pub is_dynamic: bool,
    
    /// Is statically linked
    pub is_static: bool,
    
    /// Interpreter path (for dynamic ELF)
    pub interpreter: Option<String>,
}

/// ELF architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfArchitecture {
    /// x86 32-bit
    X86,
    
    /// x86-64
    X86_64,
    
    /// ARM 32-bit
    Arm,
    
    /// ARM 64-bit (AArch64)
    Aarch64,
    
    /// RISC-V 32-bit
    RiscV32,
    
    /// RISC-V 64-bit
    RiscV64,
    
    /// WebAssembly
    Wasm,
    
    /// Unknown architecture
    Unknown,
}

/// ELF program header
#[derive(Debug)]
pub struct ElfProgramHeader {
    /// Type
    pub p_type: u32,
    
    /// Flags
    pub flags: u32,
    
    /// Offset
    pub offset: u64,
    
    /// Virtual address
    pub vaddr: u64,
    
    /// Physical address
    pub paddr: u64,
    
    /// File size
    pub file_size: u64,
    
    /// Memory size
    pub mem_size: u64,
    
    /// Alignment
    pub align: u64,
}

/// ELF section header
#[derive(Debug)]
pub struct ElfSectionHeader {
    /// Section name
    pub name: String,
    
    /// Type
    pub sh_type: u32,
    
    /// Flags
    pub flags: u64,
    
    /// Address
    pub addr: u64,
    
    /// Offset
    pub offset: u64,
    
    /// Size
    pub size: u64,
    
    /// Link
    pub link: u32,
    
    /// Info
    pub info: u32,
    
    /// Alignment
    pub align: u64,
    
    /// Entry size
    pub entry_size: u64,
}

/// Initialize the ELF loader
pub fn init() -> Result<()> {
    info!("Initializing ELF binary loader");
    
    // Create necessary directories
    let linux_dir = PathBuf::from(constants::ROOT_DIR).join(".linux");
    let loader_dir = linux_dir.join("loader");
    std::fs::create_dir_all(&loader_dir)?;
    
    info!("ELF binary loader initialized successfully");
    Ok(())
}

/// Shutdown the ELF loader
pub fn shutdown() -> Result<()> {
    info!("Shutting down ELF binary loader");
    // Nothing to do specifically
    info!("ELF binary loader shutdown complete");
    Ok(())
}

/// Parse and analyze an ELF binary
pub fn analyze_elf(path: &Path) -> Result<ElfInfo> {
    debug!("Analyzing ELF binary: {:?}", path);
    
    // Read the file
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open ELF file: {:?}", path))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .with_context(|| format!("Failed to read ELF file: {:?}", path))?;
    
    // Parse the ELF binary using goblin
    match Object::parse(&buffer)? {
        Object::Elf(elf) => {
            // Determine architecture
            let arch = match elf.header.e_machine {
                header::EM_386 => ElfArchitecture::X86,
                header::EM_X86_64 => ElfArchitecture::X86_64,
                header::EM_ARM => ElfArchitecture::Arm,
                header::EM_AARCH64 => ElfArchitecture::Aarch64,
                header::EM_RISCV => {
                    if elf.is_64 {
                        ElfArchitecture::RiscV64
                    } else {
                        ElfArchitecture::RiscV32
                    }
                },
                _ => ElfArchitecture::Unknown,
            };
            
            // Extract program headers
            let program_headers = elf.program_headers.iter().map(|ph| {
                ElfProgramHeader {
                    p_type: ph.p_type,
                    flags: ph.p_flags,
                    offset: ph.p_offset,
                    vaddr: ph.p_vaddr,
                    paddr: ph.p_paddr,
                    file_size: ph.p_filesz,
                    mem_size: ph.p_memsz,
                    align: ph.p_align,
                }
            }).collect::<Vec<_>>();
            
            // Extract section headers
            let section_headers = elf.section_headers.iter().map(|sh| {
                let name = elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("").to_string();
                ElfSectionHeader {
                    name,
                    sh_type: sh.sh_type,
                    flags: sh.sh_flags,
                    addr: sh.sh_addr,
                    offset: sh.sh_offset,
                    size: sh.sh_size,
                    link: sh.sh_link,
                    info: sh.sh_info,
                    align: sh.sh_addralign,
                    entry_size: sh.sh_entsize,
                }
            }).collect::<Vec<_>>();
            
            // Check if it's an executable
            let is_executable = elf.header.e_type == header::ET_EXEC;
            
            // Check if it's dynamically linked
            let is_dynamic = elf.libraries.len() > 0 || elf.dynamic.is_some();
            
            // Check if it's statically linked
            let is_static = !is_dynamic;
            
            // Extract interpreter path if present
            let mut interpreter = None;
            for ph in &elf.program_headers {
                if ph.p_type == goblin::elf::program_header::PT_INTERP {
                    let offset = ph.p_offset as usize;
                    let size = ph.p_filesz as usize;
                    
                    if offset + size <= buffer.len() {
                        // Extract interpreter path (null-terminated string)
                        let mut path_bytes = Vec::new();
                        for i in 0..size-1 { // -1 to skip null terminator
                            let byte = buffer[offset + i];
                            if byte == 0 {
                                break;
                            }
                            path_bytes.push(byte);
                        }
                        
                        if let Ok(path) = String::from_utf8(path_bytes) {
                            interpreter = Some(path);
                        }
                    }
                }
            }
            
            // Create ELF info
            let info = ElfInfo {
                path: path.to_path_buf(),
                arch,
                entry_point: elf.header.e_entry,
                shared_libs: elf.libraries.iter().map(|&lib| lib.to_string()).collect(),
                program_headers,
                section_headers,
                is_executable,
                is_dynamic,
                is_static,
                interpreter,
            };
            
            Ok(info)
        },
        _ => Err(anyhow!("Not an ELF binary: {:?}", path)),
    }
}

/// Check if a file is a valid ELF binary
pub fn is_elf_binary(path: &Path) -> Result<bool> {
    // Open the file
    let mut file = File::open(path)?;
    
    // Read the ELF magic number (ELFMAG)
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    
    // Check for ELF magic number: 0x7F, 'E', 'L', 'F'
    Ok(magic[0] == 0x7F && magic[1] == b'E' && magic[2] == b'L' && magic[3] == b'F')
}

/// Print information about an ELF binary
pub fn print_elf_info(info: &ElfInfo) {
    println!("ELF Binary: {:?}", info.path);
    println!("Architecture: {:?}", info.arch);
    println!("Entry Point: 0x{:x}", info.entry_point);
    println!("Type: {}", if info.is_executable { "Executable" } else { "Library" });
    println!("Linking: {}", if info.is_dynamic { "Dynamic" } else { "Static" });
    
    if let Some(ref interpreter) = info.interpreter {
        println!("Interpreter: {}", interpreter);
    }
    
    if !info.shared_libs.is_empty() {
        println!("Shared Libraries:");
        for lib in &info.shared_libs {
            println!("  {}", lib);
        }
    }
    
    println!("Program Headers: {}", info.program_headers.len());
    println!("Section Headers: {}", info.section_headers.len());
}

/// Get the executable loader for a specific architecture
fn get_loader_for_arch(arch: ElfArchitecture) -> Result<PathBuf> {
    let linux_dir = PathBuf::from(constants::ROOT_DIR).join(".linux");
    let loader_dir = linux_dir.join("loader");
    
    let loader_name = match arch {
        ElfArchitecture::X86 => "ld-linux.so.2",
        ElfArchitecture::X86_64 => "ld-linux-x86-64.so.2",
        ElfArchitecture::Arm => "ld-linux-armhf.so.3",
        ElfArchitecture::Aarch64 => "ld-linux-aarch64.so.1",
        ElfArchitecture::RiscV64 => "ld-linux-riscv64-lp64d.so.1",
        _ => return Err(anyhow!("Unsupported architecture: {:?}", arch)),
    };
    
    let loader_path = loader_dir.join(loader_name);
    
    if !loader_path.exists() {
        return Err(anyhow!("Loader not found for architecture: {:?}", arch));
    }
    
    Ok(loader_path)
}

/// Execute an ELF binary
pub fn execute_elf(path: &Path, args: &[&str]) -> Result<String> {
    info!("Executing ELF binary: {:?}", path);
    
    // Analyze the ELF binary
    let elf_info = analyze_elf(path)?;
    
    // Choose execution method based on ELF type
    if elf_info.is_dynamic {
        // For dynamically linked binaries
        if let Some(interpreter) = &elf_info.interpreter {
            // Use the specified interpreter
            let interpreter_path = PathBuf::from(interpreter);
            
            if interpreter_path.exists() {
                // Execute using the interpreter
                let mut full_args = Vec::new();
                full_args.push(interpreter.as_str());
                full_args.push(path.to_str().unwrap_or(""));
                full_args.extend(args.iter());
                
                compatibility::run_elf(&interpreter_path, &full_args.iter().copied().collect::<Vec<_>>())
            } else {
                // Try to use our loader
                let loader = get_loader_for_arch(elf_info.arch)?;
                
                let mut full_args = Vec::new();
                full_args.push(loader.to_str().unwrap_or(""));
                full_args.push(path.to_str().unwrap_or(""));
                full_args.extend(args.iter());
                
                compatibility::run_elf(&loader, &full_args.iter().copied().collect::<Vec<_>>())
            }
        } else {
            // No interpreter specified, try direct execution
            compatibility::run_elf(path, args)
        }
    } else {
        // For statically linked binaries, execute directly
        compatibility::run_elf(path, args)
    }
}

/// Execute an ELF binary inside a container
pub fn execute_elf_in_container(path: &Path, args: &[&str], container_name: &str) -> Result<String> {
    info!("Executing ELF binary in container {}: {:?}", container_name, path);
    
    // Analyze the ELF binary
    let elf_info = analyze_elf(path)?;
    
    // Similar to execute_elf, but use compatibility::run_elf_in_container
    if elf_info.is_dynamic {
        // For dynamically linked binaries
        if let Some(interpreter) = &elf_info.interpreter {
            // Use the specified interpreter
            let interpreter_path = PathBuf::from(interpreter);
            
            if interpreter_path.exists() {
                // Execute using the interpreter
                let mut full_args = Vec::new();
                full_args.push(interpreter.as_str());
                full_args.push(path.to_str().unwrap_or(""));
                full_args.extend(args.iter());
                
                compatibility::run_elf_in_container(&interpreter_path, &full_args.iter().copied().collect::<Vec<_>>(), container_name)
            } else {
                // Try to use our loader
                let loader = get_loader_for_arch(elf_info.arch)?;
                
                let mut full_args = Vec::new();
                full_args.push(loader.to_str().unwrap_or(""));
                full_args.push(path.to_str().unwrap_or(""));
                full_args.extend(args.iter());
                
                compatibility::run_elf_in_container(&loader, &full_args.iter().copied().collect::<Vec<_>>(), container_name)
            }
        } else {
            // No interpreter specified, try direct execution
            compatibility::run_elf_in_container(path, args, container_name)
        }
    } else {
        // For statically linked binaries, execute directly
        compatibility::run_elf_in_container(path, args, container_name)
    }
}

/// Load shared libraries needed by an ELF binary
pub fn load_shared_libraries(elf_info: &ElfInfo) -> Result<Vec<PathBuf>> {
    info!("Loading shared libraries for: {:?}", elf_info.path);
    
    let mut loaded_libs = Vec::new();
    
    // Standard library search paths
    let search_paths = vec![
        PathBuf::from("/lib"),
        PathBuf::from("/usr/lib"),
        PathBuf::from(constants::ROOT_DIR).join(".linux").join("lib"),
    ];
    
    // Try to find and load each required library
    for lib_name in &elf_info.shared_libs {
        let mut lib_path = None;
        
        // Search in standard paths
        for search_path in &search_paths {
            let potential_path = search_path.join(lib_name);
            if potential_path.exists() {
                lib_path = Some(potential_path);
                break;
            }
        }
        
        if let Some(path) = lib_path {
            debug!("Found shared library: {:?}", path);
            loaded_libs.push(path);
        } else {
            warn!("Could not find shared library: {}", lib_name);
        }
    }
    
    Ok(loaded_libs)
}
