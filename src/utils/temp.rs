//! Temporary file utilities using std only
//! Replaces `tempfile` crate

use std::path::PathBuf;

/// Get the system temp directory
pub fn temp_dir() -> PathBuf {
    std::env::temp_dir()
}

/// Create a temporary file path with a unique name
pub fn create_temp_file(prefix: &str, suffix: &str) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    
    let pid = std::process::id();
    
    temp_dir().join(format!("{}_{}_{}{}", prefix, pid, timestamp, suffix))
}

/// Create a temporary directory with a unique name
pub fn create_temp_dir(prefix: &str) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    
    let pid = std::process::id();
    
    temp_dir().join(format!("{}_{}_{}", prefix, pid, timestamp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_dir_exists() {
        let dir = temp_dir();
        assert!(dir.exists());
    }

    #[test]
    fn test_create_temp_file_path() {
        let path = create_temp_file("test", ".tmp");
        assert!(path.starts_with(temp_dir()));
        assert!(path.file_name().unwrap().to_str().unwrap().starts_with("test"));
    }

    #[test]
    fn test_create_temp_dir_path() {
        let path = create_temp_dir("test_dir");
        assert!(path.starts_with(temp_dir()));
        assert!(path.file_name().unwrap().to_str().unwrap().starts_with("test_dir"));
    }
}
