//! Download a file from s3 and store the contents
//! in a buffer (``Vec<u8>``) with the
//! ``s3_download_to_memory()`` function
//!
use rusoto_core::Region;
use rusoto_s3::GetObjectRequest;
use rusoto_s3::S3Client;
use rusoto_s3::S3;

use tokio::io::AsyncReadExt;

/// s3_download_to_memory
///
/// download an s3 key and return it as ``Vec[u8]``
///
/// credit to source:
/// <https://github.com/rusoto/rusoto/blob/master/integration_tests/tests/s3.rs#L903-L920>
///
/// # Arguments
///
/// * `bucket` - &str - source bucket
/// * `key` - &str - source key location
///
/// # Returns
///
/// Ok(``Vec<u8>``)
///
/// # Errors
///
/// ``String`` error messages can be returned for many reasons
/// (connectivity, aws credentials, mfa timeouts, etc.)
///
/// Err(err_msg: ``String``)
///
pub async fn s3_download_to_memory(
    bucket: &str,
    key: &str,
) -> Result<Vec<u8>, String> {
    let client = S3Client::new(Region::UsEast2);
    let get_req = GetObjectRequest {
        bucket: String::from(bucket),
        key: String::from(key),
        ..Default::default()
    };

    info!("s3_download_to_memory s3://{bucket}/{key}");
    let down_res = match client.get_object(get_req).await {
        Ok(success_res) => success_res,
        Err(_) => {
            return Err(format!("failed to download s3://{bucket}/{key}"));
        }
    };

    if false {
        trace!(
            "s3_download_to_memory(\
            {bucket}, {key}) => object down_res: {:#?}",
            down_res
        );
    }

    // https://github.com/rusoto/rusoto/blob/master/integration_tests/tests/s3.rs#L922-L940
    let mut stream = down_res.body.unwrap().into_async_read();
    let mut s3_contents = Vec::new();
    stream.read_to_end(&mut s3_contents).await.unwrap();

    Ok(s3_contents)
}
