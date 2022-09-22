//! Module for asynchronously writing a ``Vec<u8>`` to a local file
//!
use log::error;
use log::info;
use log::trace;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

/// write_buf_to_file
///
/// async file write function
///
/// # Arguments
///
/// * `file_path` - read this file path on disk
///
/// # Returns
///
/// `Vec<u8>` containing file contents as bytes
///
/// # Examples
///
/// ```rust
/// use restapi::utils::file_io::write_buf_to_file::write_buf_to_file;
/// let file_path = "/tmp/test-write-file.txt";
/// let test_string = format!("write_buf_to_file");
/// let buf: Vec<u8> = test_string.as_bytes().to_vec();
/// tokio_test::block_on(
///     write_buf_to_file(
///         file_path,
///         &buf,
///         false)
///     );
/// assert!(std::fs::metadata(&file_path).is_ok());
/// ```
///
pub async fn write_buf_to_file(
    file_path: &str,
    buf: &Vec<u8>,
    overwrite: bool,
) -> bool {
    if !overwrite && std::fs::metadata(&file_path).is_ok() {
        error!(
            "write_buf_to_file - file already exists: {file_path} \
            not overwriting"
        );
        return false;
    }
    trace!("write_buf_to_file - creating {file_path}");
    // https://stackoverflow.com/questions/49983101/serialization-of-large-struct-to-disk-with-serde-and-bincode-is-slow
    let mut file_buf_writer = BufWriter::new(File::create(file_path).unwrap());
    file_buf_writer.write_all(buf).unwrap();
    if std::fs::metadata(&file_path).is_err() {
        error!("write_buf_to_file - failed to save file: {file_path}");
        return false;
    }
    info!(
        "write_buf_to_file - wrote {} bytes to {file_path}",
        buf.len()
    );
    true
}
