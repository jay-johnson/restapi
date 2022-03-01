use std::fs;

/// path_exists
///
/// wrapper for checking if a file path exists on disk
/// 
/// # Arguments
///
/// * `path` - `&str` - file path on disk
///
/// # Examples
///
/// ```rust
/// use restapi::utils::path_exists::path_exists;
/// assert_eq!(path_exists("/tmp/i-exist"), false); 
/// ```
pub fn path_exists(
    path: &str)
-> bool {
    fs::metadata(path).is_ok()
}
