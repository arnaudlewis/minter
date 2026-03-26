use std::io;
use std::path::Path;

/// Make a path relative to the given base directory.
///
/// If the path is not under `base`, returns the original path as a string.
pub fn make_relative(path: &Path, base: &Path) -> String {
    match path.strip_prefix(base) {
        Ok(rel) => rel.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

/// Maximum file size allowed for reading (10 MB).
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Errors returned by [`read_file_safe`].
pub enum ReadError {
    NotFound,
    PermissionDenied,
    TooLarge,
    Io(io::Error),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::NotFound => write!(f, "No such file or directory"),
            ReadError::PermissionDenied => write!(f, "permission denied"),
            ReadError::TooLarge => write!(f, "exceeds maximum file size of 10MB"),
            ReadError::Io(e) => write!(f, "{}", e),
        }
    }
}

/// Read a file with a size guard (max 10 MB).
///
/// Returns a human-readable [`ReadError`] so callers can format their
/// own error messages while still getting structured error kinds.
pub fn read_file_safe(path: &Path) -> Result<String, ReadError> {
    let metadata = std::fs::metadata(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => ReadError::NotFound,
        io::ErrorKind::PermissionDenied => ReadError::PermissionDenied,
        _ => ReadError::Io(e),
    })?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(ReadError::TooLarge);
    }
    std::fs::read_to_string(path).map_err(|e| match e.kind() {
        io::ErrorKind::PermissionDenied => ReadError::PermissionDenied,
        _ => ReadError::Io(e),
    })
}
