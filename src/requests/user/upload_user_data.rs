//! ## Upload a file asynchronously to AWS S3 and store a tracking record in the db
//!
//! Upload a local file on disk to AWS S3 asynchronously and store a tracking record in the ``users_data`` table. The documentation refers to this as a ``user data`` or ``user data file`` record.
//!
//! - URL path: ``/user/data``
//! - Method: ``POST``
//! - Handler: [`upload_user_data`](crate::requests::user::upload_user_data::upload_user_data)
//! - Request: [`ApiReqUserUploadData`](crate::requests::user::upload_user_data::ApiReqUserUploadData)
//! - Response: [`ApiResUserUploadData`](crate::requests::user::upload_user_data::ApiResUserUploadData)
//!

use std::convert::Infallible;

use postgres::Row as data_row;

use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use hyper::body;
use hyper::Body;
use hyper::Response;
use hyper::HeaderMap;
use hyper::header::HeaderValue;

use serde::Serialize;
use serde::Deserialize;

use crate::core::core_config::CoreConfig;

use crate::requests::auth::validate_user_token::validate_user_token;

use crate::utils::get_uuid::get_uuid;

use crate::is3::is3::s3_upload_buffer;

/// ApiReqUserUploadData
///
/// # Request Type For upload_user_data
///
/// Handles creating a `users_data` record in the db and
/// uploading the POST-ed file contents
/// to a remote, db-tracked s3 location (`sloc`).
///
/// This type contains the uploaded file in a `Vec<u8>`
/// from the contents in a local file.
///
/// This type is the deserialized input for:
/// [`upload_user_data`](crate::requests::user::upload_user_data::upload_user_data]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// [`upload_user_data`](crate::requests::user::upload_user_data::upload_user_data)
/// function.
///
/// # Arguments
///
/// * `data` - `Vec<u8>` - contents from the POST-ed file
///   as a vector of bytes
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserUploadData {
    pub data: Vec<u8>,
}

/// ApiResUserUploadData
///
/// # Response type for upload_user_data
///
/// Return the created `users_data` db record
/// including the remote s3 location (`sloc`)
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`upload_user_data`](crate::requests::user::upload_user_data::upload_user_data]
/// and contained within the
/// hyper [`Body`](hyper::Body)
/// of the
/// hyper [`Response`](hyper::Response)
/// sent back to the client.
///
/// * `user_id` - `i32` - `users.id`
/// * `data_id` - `i32` - `users_data.id`
/// * `filename` - `String` - name of the file
/// * `data_type` - `String` - data type for the file
/// * `size_in_bytes` - `i64` - size of the file
///   (number of bytes in the POST-ed `data`)
/// * `comments` - `String` - notes or description
/// * `encoding` - `String` - encoding
/// * `sloc` - `String` - remote s3 location
/// * `msg` - `String` - help message
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResUserUploadData {
    pub user_id: i32,
    pub data_id: i32,
    pub filename: String,
    pub data_type: String,
    // https://www.google.com/search?q=rust+bigint+postgres
    // postgres size_in_bytes field is a BIGINT type
    pub size_in_bytes: i64,
    pub comments: String,
    pub encoding: String,
    pub sloc: String,
    pub msg: String,
}

/// upload_user_data
///
/// Handles uploading a POST-ed file to s3 and
/// create a `users_data` record to track the s3 utilization.
///
/// # Usage
///
/// ## Environment variables
///
/// ### Change the s3 bucket for file uploads
///
/// ```bash
/// export S3_DATA_BUCKET=BUCKET_NAME
/// ```
///
/// ### Change the s3 bucket prefix path for file uploads
///
/// ```bash
/// export S3_DATA_PREFIX="user/data/file"
/// ```
///
/// The file contents must be passed in the `data` field of the
/// [`ApiReqUserUploadData`](crate::requests::user::upload_user_data::ApiReqUserUploadData)
/// type which is serialized within a POST-ed hyper
/// [`Request`](hyper::Request)'s [`Body`](hyper::Body)
///
/// ## Overview Notes
///
/// This function only creates 1 `users_data` record at a time.
///
/// It also uploads the `data` (file contents) with a user-and-date
/// pathing convention.
///
/// # Arguments
///
/// * `tracking_label` - `&str` - caller logging label
/// * `config` - [`CoreConfig`](crate::core::core_config::CoreConfig)
/// * `db_pool` - [`Pool`](bb8::Pool) - postgres client
///   db threadpool with required tls encryption
/// * `headers` - [`HeaderMap`](hyper::HeaderMap) -
///   hashmap containing headers in key-value pairs
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
/// * `body` - `hyper::Body` - the hyper
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
///   containing the file's contents to store on s3. The
///   contents must be in the POST-ed Body's `data` key.
///
/// # Returns
///
/// ## upload_user_data on Success Returns
///
/// The newly-uploaded `users_data` record in the db
/// ([`ApiResUserUploadData`](crate::requests::user::upload_user_data::upload_user_data))
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserUploadData`](crate::requests::user::upload_user_data::ApiResUserUploadData)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `200` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## upload_user_data on Failure Returns
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserUploadData`](crate::requests::user::upload_user_data::ApiResUserUploadData)
/// dictionary with a
/// `non-200` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn upload_user_data(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    headers: &HeaderMap<HeaderValue>,
    body: hyper::Body)
-> std::result::Result<Response<Body>, Infallible>
{
    if ! headers.contains_key("user_id") {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserUploadData {
                        user_id: -1,
                        data_id: -1,
                        filename: String::from(""),
                        data_type: String::from(""),
                        size_in_bytes: 0,
                        comments: String::from(""),
                        encoding: String::from(""),
                        sloc: String::from(""),
                        msg: format!("Missing required header 'user_id' key (i.e. curl -H 'user_id: INT'"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
    let user_id_str = headers.get("user_id").unwrap().to_str().unwrap();
    let user_id: i32 = match user_id_str.parse::<i32>() {
        Ok(user_id) => user_id,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserUploadData {
                            user_id: -1,
                            data_id: -1,
                            filename: String::from(""),
                            data_type: String::from(""),
                            size_in_bytes: 0,
                            comments: String::from(""),
                            encoding: String::from(""),
                            sloc: String::from(""),
                            msg: format!("user_id must be a postive number that is the actual user_id for the token"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };
    if ! headers.contains_key("filename") {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserUploadData {
                        user_id: -1,
                        data_id: -1,
                        filename: String::from(""),
                        data_type: String::from(""),
                        size_in_bytes: 0,
                        comments: String::from(""),
                        encoding: String::from(""),
                        sloc: String::from(""),
                        msg: format!("Missing required header 'filename' key (i.e. curl -H 'user_id: INT'"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
    let file_name_str = headers.get("filename").unwrap().to_str().unwrap();
    let file_name_len = file_name_str.len();
    if
            file_name_len < 1 || file_name_len > 511 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserUploadData {
                        user_id: -1,
                        data_id: -1,
                        filename: String::from(""),
                        data_type: String::from(""),
                        size_in_bytes: 0,
                        comments: String::from(""),
                        encoding: String::from(""),
                        sloc: String::from(""),
                        msg: format!("The header value for 'filename' must be between 1 and 511 characters"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
    // -H 'filename: testfile.txt' -H 'data_type: file' -H 'encoding: na' -H 'comments: this is a test comment' -H 'sloc: s3://bucket/prefix'
    let encoding = match headers.get("encoding") {
        Some(v) => String::from(format!("{}", v.to_str().unwrap())),
        None => String::from("na"),
    };
    let comments = match headers.get("comments") {
        Some(v) => String::from(format!("{}", v.to_str().unwrap())),
        None => String::from("file"),
    };
    let data_type = match headers.get("data_type") {
        Some(v) => String::from(format!("{}", v.to_str().unwrap())),
        None => String::from("file"),
    };
    let sloc_start = match headers.get("sloc") {
        Some(v) => String::from(format!("{}", v.to_str().unwrap())),
        None => String::from(""),
    };
    let should_upload_to_s3 = match headers.get("s3_enable") {
        Some(_) => true,
        None => std::env::var("S3_DATA_UPLOAD_TO_S3").unwrap_or(String::from("0")) == String::from("0"),
    };

    let s3_bucket = std::env::var("S3_DATA_BUCKET").unwrap_or(String::from("BUCKET_NAME"));
    let s3_prefix = std::env::var("S3_DATA_PREFIX").unwrap_or(String::from("user/data/file"));
    let now = chrono::Utc::now();
    let now_str = now.format("%Y/%m/%d");
    let s3_uuid = get_uuid();
    let s3_key_dst = format!("\
        {s3_prefix}/\
        {user_id}/\
        {now_str}/\
        {s3_uuid}.{file_name_str}");
    let sloc = match sloc_start.len() {
        0 => {
            format!("s3://{s3_bucket}/{s3_key_dst}")
        }
        _ => sloc_start,
    };

    {
        let conn = db_pool.get().await.unwrap();
        let _token = match validate_user_token(
                tracking_label,
                &config,
                &conn,
                headers,
                user_id).await {
            Ok(_token) => _token,
            Err(_) => {
                let response = Response::builder()
                    .status(400)
                    .body(Body::from(
                        serde_json::to_string(
                            &ApiResUserUploadData {
                                user_id: -1,
                                data_id: -1,
                                filename: String::from(""),
                                data_type: String::from(""),
                                size_in_bytes: 0,
                                comments: String::from(""),
                                encoding: String::from(""),
                                sloc: String::from(""),
                                msg: format!("\
                                    User data upload failed due to invalid token"),
                            }
                        ).unwrap()))
                .unwrap();
                return Ok(response);
            }
        };
    }

    info!("{tracking_label} - receiving user_id={user_id} name={file_name_str} data");
    let bytes = body::to_bytes(body).await.unwrap();
    let file_contents_size: usize = bytes.len() as usize;
    if
            file_contents_size < 1 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserUploadData {
                        user_id: -1,
                        data_id: -1,
                        filename: String::from(""),
                        data_type: String::from(""),
                        size_in_bytes: 0,
                        comments: String::from(""),
                        encoding: String::from(""),
                        sloc: String::from(""),
                        msg: format!("No data uploaded in the body"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    let file_contents_size_in_mb: f32 = file_contents_size as f32 / 1024.0 / 1024.0;

    info!("\
        {tracking_label} - processing data for user_id={user_id} \
        name={file_name_str} \
        size={file_contents_size_in_mb:.2}mb \
        upload_to_s3={should_upload_to_s3} \
        {sloc}");

    if should_upload_to_s3 {
        match s3_upload_buffer(
                    tracking_label,
                    &s3_bucket,
                    &s3_key_dst,
                    &bytes)
                .await {
            Ok(good_msg) => {
                info!("{good_msg} - done uploading - {sloc}")
            },
            Err(emsg) => {
                info!("{emsg} - failed uploading {sloc}")
            }
        }
    }
    else {
        info!("\
            {tracking_label} - not uploading to s3");
    }

    let conn = db_pool.get().await.unwrap();
    let cur_query = format!("\
        INSERT INTO \
        users_data (\
            user_id, \
            filename, \
            data_type, \
            size_in_bytes, \
            comments, \
            encoding, \
            sloc) \
        VALUES (\
            {user_id},
            '{file_name_str}',
            '{data_type}',
            {file_contents_size},
            '{comments}',
            '{encoding}',
            '{sloc}') \
        RETURNING \
            users_data.id,
            users_data.user_id,
            users_data.filename,
            users_data.data_type,
            users_data.size_in_bytes,
            users_data.comments,
            users_data.encoding,
            users_data.sloc;");
    let stmt = conn.prepare(&cur_query).await.unwrap();
    let mut query_result: Vec<data_row> = Vec::with_capacity(5);
    if false {
        println!("{}", query_result.len());
    }
    query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let err_msg = format!("{}", e);
            let response = Response::builder()
                .status(500)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserUploadData {
                            user_id: -1,
                            data_id: -1,
                            filename: String::from(""),
                            data_type: String::from(""),
                            size_in_bytes: 0,
                            comments: String::from(""),
                            encoding: String::from(""),
                            sloc: String::from(""),
                            msg: format!("User data upload failed for user_id={user_id} with err='{err_msg}'")
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };
    let mut row_list: Vec<ApiResUserUploadData> = Vec::with_capacity(1);
    for row in query_result.iter() {
        let found_data_id: i32 = row.try_get("id").unwrap();
        let found_user_id: i32 = row.try_get("user_id").unwrap();
        let found_filename: String = row.try_get("filename").unwrap();
        let found_data_type: String = row.try_get("data_type").unwrap();
        let found_size_in_bytes: i64 = row.try_get("size_in_bytes").unwrap();
        let found_comments: String = row.try_get("comments").unwrap();
        let found_encoding: String = row.try_get("encoding").unwrap();
        let found_sloc: String = row.try_get("sloc").unwrap();
        row_list.push(ApiResUserUploadData {
            user_id: found_user_id,
            data_id: found_data_id,
            filename: found_filename,
            data_type: found_data_type,
            size_in_bytes: found_size_in_bytes,
            comments: found_comments,
            encoding: found_encoding,
            sloc: found_sloc,
            msg: format!("success"),
        });
    }
    if row_list.len() == 0 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserUploadData {
                        user_id: -1,
                        data_id: -1,
                        filename: String::from(""),
                        data_type: String::from(""),
                        size_in_bytes: 0,
                        comments: String::from(""),
                        encoding: String::from(""),
                        sloc: String::from(""),
                        msg: format!("no upload data found in db"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
    else {
        let response = Response::builder()
            .status(200)
            .body(Body::from(
                serde_json::to_string(&row_list[0])
                .unwrap()))
            .unwrap();
        return Ok(response);
    }
}
