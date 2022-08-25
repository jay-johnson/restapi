use serde::Deserialize;
use serde::Serialize;

/// ModelUserData
///
/// Representation in the db for a
/// user's s3-uploaded file
///
/// Each user can store many `users_data` record(s)
///
/// # DB table
///
/// `users_data`
///
/// # Arguments
///
/// * `user_id` - `i32` - user id in the db
/// * `data_id` - `i32` - users_data.id in the db
/// * `filename` - `String` - data filename
/// * `data_type` - `String` - v2 - data type for restricting
///   upload types
/// * `size_in_bytes` - `i64` - size of the uploaded file
/// * `comments` - `String` - HTTP header can add comments to
///   the file
/// * `encoding` - `String` - file encoding
/// * `sloc` - `String` - full s3 location path
/// * `created_at` - `String` - original upload time
/// * `updated_at` - `String` - most recent update time
/// * `msg` - `String` - message for
///   helping debug from the client
///
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ModelUserData {
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
    // https://github.com/sfackler/rust-postgres/issues/498#issuecomment-541745277
    // chrono::DateTime<chrono::Utc>
    pub created_at: String,
    pub updated_at: String,
    pub msg: String,
}
