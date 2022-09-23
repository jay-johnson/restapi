//! Download a file from s3 using the
//! ``s3_download_to_file()`` function
//!
use crate::is3::s3_download_to_memory::s3_download_to_memory;
use std::fs;

/// s3_download_to_file
///
/// download a key from s3 and save it to a file
///
/// # Arguments
///
/// * `file_path` - &str - save to this file path on disk
/// * `bucket` - &str - source bucket
/// * `key` - &str - key location
///
/// # Returns
///
/// Ok(file_path: ``String``)
///
/// # Errors
///
/// ``String`` error messages can be returned for many reasons
/// (connectivity, aws credentials, mfa timeouts, etc.)
///
/// Err(err_msg: ``String``)
///
pub async fn s3_download_to_file(
    file_path: &str,
    bucket: &str,
    key: &str,
) -> Result<String, String> {
    match s3_download_to_memory(bucket, key).await {
        Ok(s3_contents) => {
            if false {
                info!(
                    "s3_download_to_file - saving - \
                    s3://{bucket}/{key} \
                    at {file_path}"
                );
            }
            fs::write(file_path, s3_contents).unwrap();
            if false {
                info!(
                    "s3_download_to_file - saved - \
                    s3://{bucket}/{key} \
                    at {file_path}"
                );
            }
            Ok(file_path.to_string())
        }
        Err(emsg) => Err(emsg),
    }
}
