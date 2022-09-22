//! Upload a local file to s3 using the function:
//! ``s3_upload_file()``
//!
use std::io::Read;
use std::sync::Arc;
use std::sync::Mutex;

// if tracing performance include this:
// use std::time::Instant;

use rusoto_core::Region;
use rusoto_s3::CompleteMultipartUploadRequest;
use rusoto_s3::CompletedMultipartUpload;
use rusoto_s3::CompletedPart;
use rusoto_s3::CreateMultipartUploadRequest;
use rusoto_s3::S3Client;
use rusoto_s3::UploadPartRequest;
use rusoto_s3::S3;

/// s3_upload_file
///
/// An async upload a local file on disk (`&str`)
/// using an `futures`, `multipart_upload` to s3
///
/// This works by reading the file into memory, then creating
/// many smaller chunks (6,000,000 bytes per chunk).
/// After creating the chunks, each one is loaded into a list of
/// `multipart` `futures` that are processed asynchronously
///
/// # Usage
///
/// Change the default s3 storage class with:
///
/// ```bash
/// export S3_STORAGE_CLASS=STANDARD
/// ```
///
/// Credit Source to:
///
/// <https://github.com/rusoto/rusoto/blob/master/integration_tests/tests/s3.rs#L903-L920>
/// <https://stackoverflow.com/questions/63553317/s3-upload-using-rusoto>
///
/// # Arguments
///
/// * `file_path` - &str - file path on disk to upload
/// * `bucket` - &str - destination bucket
/// * `key` - &str - destination key location
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
/// Err(err_msg: `String`)
///
/// # Examples
///
/// ```
/// [TODO:example]
/// ```
pub async fn s3_upload_file(
    file_path: &str,
    bucket: &str,
    key: &str,
) -> Result<String, String> {
    // if tracing - uncomment the timer here
    // let now = Instant::now();
    let mut file = std::fs::File::open(file_path).unwrap();
    let chunk_size: usize = 6_000_000;
    let mut buffer = Vec::with_capacity(chunk_size);
    let s3_bucket = String::from(bucket);
    let s3_key = String::from(key);
    let s3_bucket_copy = String::from(bucket);
    let s3_key_copy = String::from(key);
    let server_side_encryption = "AES256";
    let storage_class = std::env::var("S3_STORAGE_CLASS")
        .unwrap_or_else(|_| "STANDARD".to_string());

    let client = S3Client::new(Region::UsEast2);

    info!(
        "s3_upload_file - start - \
        {file_path} \
        to s3://{bucket}/{key} with \
        sse={server_side_encryption} \
        sc={storage_class}"
    );

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
        "s3_upload_file - waiting - \
        {file_path} \
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
                "s3_upload_file - failed to create s3 multipart upload \
                s3://{s3_bucket_copy}/{s3_key_copy} \
                with err='{e}'"
            );
            // 2) check for: access deny
            if full_err_msg.contains("<Code>AccessDenied</Code>") {
                let err_msg = format!(
                    "s3_upload_file - failed with access denied - \
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
    loop {
        let maximum_bytes_to_read = chunk_size - buffer.len();
        // println!("maximum_bytes_to_read: {}", maximum_bytes_to_read);
        file.by_ref()
            .take(maximum_bytes_to_read as u64)
            .read_to_end(&mut buffer)
            .unwrap();
        // println!("length: {}", buffer.len());
        // println!("part_number: {}", part_number);
        if buffer.is_empty() {
            // The file has ended.
            break;
        }
        let next_buffer = Vec::with_capacity(chunk_size);
        let data_to_send = buffer;
        let completed_parts_cloned = completed_parts.clone();
        let create_upload_part_arc_cloned = create_upload_part_arc.clone();
        let send_part_task_future = tokio::task::spawn(async move {
            let part = create_upload_part_arc_cloned(
                data_to_send.to_vec(),
                part_number,
            );
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
        buffer = next_buffer;
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
        trace!(
            "s3_upload_file - done - \
            {file_path} to s3://{bucket}/{key} \
            with sse={server_side_encryption} \
            sc={storage_class} \
            with stime taken: {}, with chunk:: {}",
            now.elapsed().as_secs(),
            chunk_size
        );
    }
    */
    final_client
        .complete_multipart_upload(complete_req)
        .await
        .expect("Couldn't complete multipart upload");

    info!(
        "s3_upload_file - done - \
        {file_path} to s3://{bucket}/{key} \
        with sse={server_side_encryption} \
        sc={storage_class}"
    );

    Ok("Success".to_string())
}
