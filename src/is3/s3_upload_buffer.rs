//! Upload a slice of bytes (``[u8]``) and store the contents
//! in a single s3 key (file) using the function:
//! ``s3_upload_buffer()``
//!
use std::sync::Arc;
use std::sync::Mutex;

use rusoto_core::Region;
use rusoto_s3::CompleteMultipartUploadRequest;
use rusoto_s3::CompletedMultipartUpload;
use rusoto_s3::CompletedPart;
use rusoto_s3::CreateMultipartUploadRequest;
use rusoto_s3::S3Client;
use rusoto_s3::UploadPartRequest;
use rusoto_s3::S3;

/// s3_upload_buffer
///
/// An async upload an in-memory buffer (``&[u8]``)
/// using an ``futures``, ``multipart_upload`` to s3
///
/// This works by chunking the ``bytes`` buffer (type: ``&[u8]``) into
/// many smaller chunks (6,000,000 bytes per chunk).
/// after creating the chunks, each one is loaded into a list of
/// ``multipart`` ``futures`` that are processed asynchronously.
/// Once the ``futures`` are done, the file is done uploading to s3.
///
/// # Usage
///
/// Change the default s3 storage class with:
///
/// ```bash
/// export S3_STORAGE_CLASS=STANDARD
/// ```
///
/// Credit source to:
///
/// <https://github.com/rusoto/rusoto/blob/master/integration_tests/tests/s3.rs#L903-L920>
/// <https://stackoverflow.com/questions/63553317/s3-upload-using-rusoto>
///
/// # Arguments
///
/// * `tracking_label` - &str - logging label for the caller
/// * `bucket` - &str - destination bucket
/// * `key` - &str - destination key location
/// * `bytes` - &[u8] - buffer to upload into s3
///
/// # Returns
///
/// Ok(success_msg: `String`)
///
/// # Errors
///
/// `String` error messages can be returned for many reasons
/// (connectivity, aws credentials, mfa timeouts, etc.)
///
/// Err(err_msg: `String)
///
/// # Examples
///
/// ```
/// use crate::is3::is3::s3_upload_buffer;
/// let bytes = format!("test-s3-upload-buffer")
///     .as_bytes()
///     .to_vec();
/// match s3_upload_buffer(
///         "test-s3-upload-buffer",
///         "BUCKET",
///         "PATH_TO_KEY",
///         &bytes).await {
///     Ok(good_msg) => {
///         info!("{good_msg} - done uploading to s3://{s3_bucket}/{s3_key_dst}")
///     },
///     Err(emsg) => {
///         info!("{emsg} - failed uploading to s3://{s3_bucket}/{s3_key_dst}")
///     }
/// }
/// ```
pub async fn s3_upload_buffer(
    tracking_label: &str,
    bucket: &str,
    key: &str,
    bytes: &[u8],
) -> Result<String, String> {
    // let now = Instant::now();
    let s3_bucket = String::from(bucket);
    let s3_key = String::from(key);
    let s3_bucket_copy = String::from(bucket);
    let s3_key_copy = String::from(key);
    let server_side_encryption = "AES256";
    let storage_class = std::env::var("S3_STORAGE_CLASS")
        .unwrap_or_else(|_| "STANDARD".to_string());

    let upload_size_in_bytes = bytes.len();
    let upload_size_in_mb: f32 = upload_size_in_bytes as f32 / 1024.0 / 1024.0;
    let chunk_size: usize = 6_000_000;

    let buffer: Vec<u8> = match upload_size_in_bytes > chunk_size {
        true => Vec::with_capacity(chunk_size),
        false => Vec::with_capacity(upload_size_in_bytes),
    };

    info!(
        "{tracking_label} - s3_upload_buffer - start - \
        {upload_size_in_mb:.2}mb \
        to s3://{bucket}/{key} with \
        sse={server_side_encryption} \
        sc={storage_class} \
        buffer_size={}",
        buffer.len()
    );

    let client = S3Client::new(Region::UsEast2);

    let create_multipart_request = CreateMultipartUploadRequest {
        bucket: s3_bucket,
        key: s3_key,
        server_side_encryption: Some(server_side_encryption.to_string()),
        storage_class: Some(storage_class.to_string()),
        ..Default::default()
    };

    // Start the multipart upload and note the upload_id generated
    let create_response_result = client
        .create_multipart_upload(create_multipart_request)
        .await;

    info!(
        "{tracking_label} - s3_upload_buffer - waiting - \
        {upload_size_in_mb:.2}mb \
        to s3://{bucket}/{key} with \
        sse={server_side_encryption} \
        sc={storage_class}"
    );
    // Result<CreateMultipartUploadOutput, RusotoError<CreateMultipartUploadError>>;
    let create_response = match create_response_result {
        Ok(d) => d,
        Err(e) => {
            // rusoto does not really have an api for looking at the error
            // 1) cast as a string
            let full_err_msg = format!(
                "{tracking_label} - s3_upload_buffer - \
                    failed to create s3 multipart upload \
                    s3://{s3_bucket_copy}/{s3_key_copy} \
                    with err='{e}'"
            );
            // 2) check for: access deny
            if full_err_msg.contains("<Code>AccessDenied</Code>") {
                let err_msg = format!(
                    "{tracking_label} - s3_upload_buffer - \
                    failed with access denied - \
                    please confirm the environment variables \
                    AWS_SECRET_ACCESS_KEY and AWS_ACCESS_KEY_ID \
                    are set correctly (or other aws account credentials) \
                    s3://{s3_bucket_copy}/{s3_key_copy}"
                );
                return Err(err_msg);
            }
            // fall-thru) general exception failure - need to add support for it
            else {
                return Err(full_err_msg);
            }
        }
    };
    let upload_id = create_response.upload_id.unwrap();
    let upload_id_clone = upload_id.clone();

    let s3_bucket = String::from(bucket);
    let s3_key = String::from(key);
    // Create upload parts
    let create_upload_part =
        move |body: Vec<u8>, part_number: i64| -> UploadPartRequest {
            UploadPartRequest {
                body: Some(body.into()),
                bucket: s3_bucket.clone(),
                key: s3_key.clone(),
                upload_id: upload_id_clone.to_owned(),
                part_number,
                ..Default::default()
            }
        };

    let create_upload_part_arc = Arc::new(create_upload_part);
    let completed_parts = Arc::new(Mutex::new(vec![]));

    let mut part_number = 1;

    let mut multiple_parts_futures = Vec::new();
    for buffer in bytes.chunks(chunk_size as usize) {
        /*
        info!("{tracking_label} - s3_upload_buffer - \
            chunk={part_number} - \
            {upload_size_in_mb:.2}mb to s3://{bucket}/{key}");
        */
        let data_to_send: Vec<u8> = buffer.to_vec();
        let completed_parts_cloned = completed_parts.clone();
        let create_upload_part_arc_cloned = create_upload_part_arc.clone();
        let send_part_task_future = tokio::task::spawn(async move {
            let part = create_upload_part_arc_cloned(data_to_send, part_number);
            {
                let part_number = part.part_number;
                let internal_loop_client = S3Client::new(Region::UsEast2);
                let response = internal_loop_client.upload_part(part).await;
                completed_parts_cloned.lock().unwrap().push(CompletedPart {
                    e_tag: response
                        .expect("Couldn't complete multipart upload")
                        .e_tag,
                    part_number: Some(part_number),
                });
            }
        });
        multiple_parts_futures.push(send_part_task_future);
        part_number += 1;
    }
    let final_client = S3Client::new(Region::UsEast2);
    // println!("waiting for futures");
    let _results = futures::future::join_all(multiple_parts_futures).await;

    let mut completed_parts_vector = completed_parts.lock().unwrap().to_vec();
    completed_parts_vector.sort_by_key(|part| part.part_number);
    // println!("futures done");
    let completed_upload = CompletedMultipartUpload {
        parts: Some(completed_parts_vector),
    };

    let s3_bucket = String::from(bucket);
    let s3_key = String::from(key);
    let complete_req = CompleteMultipartUploadRequest {
        bucket: s3_bucket,
        key: s3_key,
        upload_id: upload_id.to_owned(),
        multipart_upload: Some(completed_upload),
        ..Default::default()
    };

    /*
    if true {
        trace!("s3_upload_buffer - done - \
            {upload_size_in_mb:.2}mb to s3://{bucket}/{key} \
            with sse={server_side_encryption} \
            sc={storage_class} \
            with stime taken: {}, with chunk:: {}",
            now.elapsed().as_secs(),
            chunk_size);
    }
    */
    final_client
        .complete_multipart_upload(complete_req)
        .await
        .expect("Couldn't complete multipart upload");

    info!(
        "{tracking_label} - s3_upload_buffer - done - \
        {upload_size_in_mb:.2}mb to s3://{bucket}/{key} \
        with sse={server_side_encryption} \
        sc={storage_class}"
    );

    Ok("Success".to_string())
}
