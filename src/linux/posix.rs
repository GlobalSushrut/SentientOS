use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// POSIX error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosixError {
    EPERM,  // Operation not permitted
    ENOENT, // No such file or directory
    ESRCH,  // No such process
    EINTR,  // Interrupted system call
    EIO,    // I/O error
    ENXIO,  // No such device or address
    E2BIG,  // Argument list too long
    ENOEXEC,// Exec format error
    EBADF,  // Bad file number
    ECHILD, // No child processes
    EAGAIN, // Try again
    ENOMEM, // Out of memory
    EACCES, // Permission denied
    EFAULT, // Bad address
    ENOTBLK,// Block device required
    EBUSY,  // Device or resource busy
    EEXIST, // File exists
    EXDEV,  // Cross-device link
    ENODEV, // No such device
    ENOTDIR,// Not a directory
    EISDIR, // Is a directory
    EINVAL, // Invalid argument
    ENFILE, // File table overflow
    EMFILE, // Too many open files
    ENOTTY, // Not a typewriter
    ETXTBSY,// Text file busy
    EFBIG,  // File too large
    ENOSPC, // No space left on device
    ESPIPE, // Illegal seek
    EROFS,  // Read-only file system
    EMLINK, // Too many links
    EPIPE,  // Broken pipe
    EDOM,   // Math argument out of domain
    ERANGE, // Math result not representable
    ENOSYS, // Function not implemented
}

// Global file descriptor table
lazy_static::lazy_static! {
    static ref FD_TABLE: Arc<Mutex<HashMap<i32, FileDescriptor>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

/// File descriptor type
#[derive(Debug, Clone)]
struct FileDescriptor {
    /// File descriptor number
    fd: i32,
    
    /// Path to the file
    path: String,
    
    /// File mode (read, write, etc.)
    mode: FileMode,
    
    /// File offset
    offset: u64,
}

/// File mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileMode {
    /// Read only
    Read,
    
    /// Write only
    Write,
    
    /// Read and write
    ReadWrite,
}

/// Initialize the POSIX compatibility layer
pub fn init() -> Result<()> {
    info!("Initializing POSIX compatibility layer");
    
    // Create necessary directories
    let posix_dir = PathBuf::from(crate::core::constants::ROOT_DIR)
        .join(".linux")
        .join("posix");
    
    std::fs::create_dir_all(&posix_dir)
        .context("Failed to create POSIX directory")?;
    
    // Initialize the standard file descriptors
    initialize_standard_fds()?;
    
    info!("POSIX compatibility layer initialized successfully");
    Ok(())
}

/// Shutdown the POSIX compatibility layer
pub fn shutdown() -> Result<()> {
    info!("Shutting down POSIX compatibility layer");
    
    // Close all open file descriptors
    let mut fd_table = FD_TABLE.lock().unwrap();
    fd_table.clear();
    
    info!("POSIX compatibility layer shutdown complete");
    Ok(())
}

/// Initialize standard file descriptors (stdin, stdout, stderr)
fn initialize_standard_fds() -> Result<()> {
    debug!("Initializing standard file descriptors");
    
    let mut fd_table = FD_TABLE.lock().unwrap();
    
    // Initialize stdin (fd 0)
    fd_table.insert(0, FileDescriptor {
        fd: 0,
        path: "/dev/stdin".to_string(),
        mode: FileMode::Read,
        offset: 0,
    });
    
    // Initialize stdout (fd 1)
    fd_table.insert(1, FileDescriptor {
        fd: 1,
        path: "/dev/stdout".to_string(),
        mode: FileMode::Write,
        offset: 0,
    });
    
    // Initialize stderr (fd 2)
    fd_table.insert(2, FileDescriptor {
        fd: 2,
        path: "/dev/stderr".to_string(),
        mode: FileMode::Write,
        offset: 0,
    });
    
    debug!("Standard file descriptors initialized");
    Ok(())
}

/// Open a file
pub fn open(path: &str, flags: i32, mode: i32) -> Result<i32> {
    debug!("POSIX open: path={}, flags={:#x}, mode={:#o}", path, flags, mode);
    
    // Translate the path
    let sys_path = super::filesystem::translate_to_linux_path(path);
    
    // Determine file mode
    let file_mode = if (flags & 0x02) != 0 {
        // O_RDWR
        FileMode::ReadWrite
    } else if (flags & 0x01) != 0 {
        // O_WRONLY
        FileMode::Write
    } else {
        // O_RDONLY (default)
        FileMode::Read
    };
    
    // Create file if O_CREAT flag is set
    if (flags & 0x40) != 0 {
        // Check if file exists
        if !std::path::Path::new(&sys_path).exists() {
            // Create file
            std::fs::File::create(&sys_path)
                .with_context(|| format!("Failed to create file: {}", sys_path))?;
        }
    }
    
    // Check if file exists
    if !std::path::Path::new(&sys_path).exists() {
        return Err(anyhow::anyhow!("File not found: {}", path));
    }
    
    // Allocate a new file descriptor
    let mut fd_table = FD_TABLE.lock().unwrap();
    
    // Find the next available file descriptor
    let mut new_fd = 3; // Start after standard FDs
    while fd_table.contains_key(&new_fd) {
        new_fd += 1;
    }
    
    // Create and insert the file descriptor
    fd_table.insert(new_fd, FileDescriptor {
        fd: new_fd,
        path: sys_path,
        mode: file_mode,
        offset: 0,
    });
    
    debug!("Allocated file descriptor: {}", new_fd);
    Ok(new_fd)
}

/// Close a file
pub fn close(fd: i32) -> Result<()> {
    debug!("POSIX close: fd={}", fd);
    
    // Remove the file descriptor from the table
    let mut fd_table = FD_TABLE.lock().unwrap();
    
    if fd_table.remove(&fd).is_some() {
        debug!("Closed file descriptor: {}", fd);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid file descriptor: {}", fd))
    }
}

/// Read from a file
pub fn read(fd: i32, buf: &mut [u8]) -> Result<usize> {
    debug!("POSIX read: fd={}, buf_len={}", fd, buf.len());
    
    let mut fd_table = FD_TABLE.lock().unwrap();
    
    // Get the file descriptor
    let file_desc = fd_table.get_mut(&fd)
        .ok_or_else(|| anyhow::anyhow!("Invalid file descriptor: {}", fd))?;
    
    // Check if the file is readable
    if file_desc.mode == FileMode::Write {
        return Err(anyhow::anyhow!("File not open for reading: {}", fd));
    }
    
    // Handle standard input specially
    if fd == 0 {
        // In a real implementation, this would read from stdin
        // For now, we'll just return some example data
        let example = b"example input\n";
        let len = example.len().min(buf.len());
        buf[..len].copy_from_slice(&example[..len]);
        return Ok(len);
    }
    
    // Read from the file
    let mut file = std::fs::File::open(&file_desc.path)
        .with_context(|| format!("Failed to open file: {}", file_desc.path))?;
    
    // Seek to the current offset
    use std::io::{Read, Seek, SeekFrom};
    file.seek(SeekFrom::Start(file_desc.offset))?;
    
    // Read data
    let bytes_read = file.read(buf)?;
    
    // Update the offset
    file_desc.offset += bytes_read as u64;
    
    debug!("Read {} bytes from fd {}", bytes_read, fd);
    Ok(bytes_read)
}

/// Write to a file
pub fn write(fd: i32, buf: &[u8]) -> Result<usize> {
    debug!("POSIX write: fd={}, buf_len={}", fd, buf.len());
    
    let mut fd_table = FD_TABLE.lock().unwrap();
    
    // Get the file descriptor
    let file_desc = fd_table.get_mut(&fd)
        .ok_or_else(|| anyhow::anyhow!("Invalid file descriptor: {}", fd))?;
    
    // Check if the file is writable
    if file_desc.mode == FileMode::Read {
        return Err(anyhow::anyhow!("File not open for writing: {}", fd));
    }
    
    // Handle standard output and error specially
    if fd == 1 || fd == 2 {
        // In a real implementation, this would write to stdout/stderr
        // For now, we'll just log the data
        if let Ok(s) = std::str::from_utf8(buf) {
            if fd == 1 {
                info!("stdout: {}", s.trim_end());
            } else {
                warn!("stderr: {}", s.trim_end());
            }
        }
        return Ok(buf.len());
    }
    
    // Write to the file
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(&file_desc.path)
        .with_context(|| format!("Failed to open file for writing: {}", file_desc.path))?;
    
    // Seek to the current offset
    use std::io::{Write, Seek, SeekFrom};
    file.seek(SeekFrom::Start(file_desc.offset))?;
    
    // Write data
    let bytes_written = file.write(buf)?;
    
    // Update the offset
    file_desc.offset += bytes_written as u64;
    
    debug!("Wrote {} bytes to fd {}", bytes_written, fd);
    Ok(bytes_written)
}

/// Seek in a file
pub fn lseek(fd: i32, offset: i64, whence: i32) -> Result<u64> {
    debug!("POSIX lseek: fd={}, offset={}, whence={}", fd, offset, whence);
    
    let mut fd_table = FD_TABLE.lock().unwrap();
    
    // Get the file descriptor
    let file_desc = fd_table.get_mut(&fd)
        .ok_or_else(|| anyhow::anyhow!("Invalid file descriptor: {}", fd))?;
    
    // Calculate the new offset based on whence
    let new_offset = match whence {
        0 => { // SEEK_SET
            if offset < 0 {
                return Err(anyhow::anyhow!("Invalid offset for SEEK_SET: {}", offset));
            }
            offset as u64
        },
        1 => { // SEEK_CUR
            let current = file_desc.offset;
            if offset < 0 && current < (-offset) as u64 {
                return Err(anyhow::anyhow!("Invalid offset for SEEK_CUR: {}", offset));
            }
            if offset < 0 {
                current - (-offset) as u64
            } else {
                current + offset as u64
            }
        },
        2 => { // SEEK_END
            // Get the file size
            let metadata = std::fs::metadata(&file_desc.path)
                .with_context(|| format!("Failed to get file metadata: {}", file_desc.path))?;
            
            let size = metadata.len();
            if offset < 0 && size < (-offset) as u64 {
                return Err(anyhow::anyhow!("Invalid offset for SEEK_END: {}", offset));
            }
            if offset < 0 {
                size - (-offset) as u64
            } else {
                size + offset as u64
            }
        },
        _ => {
            return Err(anyhow::anyhow!("Invalid whence value: {}", whence));
        }
    };
    
    // Update the offset
    file_desc.offset = new_offset;
    
    debug!("Seeked fd {} to offset {}", fd, new_offset);
    Ok(new_offset)
}

/// Convert POSIX error to Linux errno
pub fn posix_error_to_errno(error: PosixError) -> i32 {
    match error {
        PosixError::EPERM => 1,
        PosixError::ENOENT => 2,
        PosixError::ESRCH => 3,
        PosixError::EINTR => 4,
        PosixError::EIO => 5,
        PosixError::ENXIO => 6,
        PosixError::E2BIG => 7,
        PosixError::ENOEXEC => 8,
        PosixError::EBADF => 9,
        PosixError::ECHILD => 10,
        PosixError::EAGAIN => 11,
        PosixError::ENOMEM => 12,
        PosixError::EACCES => 13,
        PosixError::EFAULT => 14,
        PosixError::ENOTBLK => 15,
        PosixError::EBUSY => 16,
        PosixError::EEXIST => 17,
        PosixError::EXDEV => 18,
        PosixError::ENODEV => 19,
        PosixError::ENOTDIR => 20,
        PosixError::EISDIR => 21,
        PosixError::EINVAL => 22,
        PosixError::ENFILE => 23,
        PosixError::EMFILE => 24,
        PosixError::ENOTTY => 25,
        PosixError::ETXTBSY => 26,
        PosixError::EFBIG => 27,
        PosixError::ENOSPC => 28,
        PosixError::ESPIPE => 29,
        PosixError::EROFS => 30,
        PosixError::EMLINK => 31,
        PosixError::EPIPE => 32,
        PosixError::EDOM => 33,
        PosixError::ERANGE => 34,
        PosixError::ENOSYS => 38,
    }
}
