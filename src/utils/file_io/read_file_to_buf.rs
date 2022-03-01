use log::info;
use log::trace;
use log::error;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;

/// read_file_to_buf
///
/// async file read function
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
/// use restapi::utils::file_io::read_file_to_buf::read_file_to_buf;
/// let file_bytes = tokio_test::block_on(
///     read_file_to_buf(
///         "./README.md")
///     );
/// assert!(file_bytes.len() > 0);
/// ```
///
pub async fn read_file_to_buf(
    file_path: &str)
-> Vec<u8>
{
    let mut buf: Vec<u8> = Vec::new();
    if ! std::fs::metadata(&file_path).is_ok() {
        error!("read_file_to_buf - file does not exist: {file_path}");
        return buf;
    }
    trace!("read_file_to_buf - file_path={file_path}");
    let loc_file_handle = File::open(file_path).expect("Unable to open file");
    let mut buf_reader_o = BufReader::new(loc_file_handle);
    buf_reader_o.read_to_end(&mut buf).expect("Unable to read string");
    info!("\
        read_file_to_buf -  {} bytes from file {file_path}",
        buf.len());
    return buf;
}
