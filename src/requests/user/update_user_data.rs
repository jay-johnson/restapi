//! ## Update an existing user data file record for a file stored in AWS S3
//!
//! Update the ``users_data`` tracking record for a file that exists in AWS S3
//!
//! - URL path: ``/user/data``
//! - Method: ``PUT``
//! - Handler: [`update_user_data`](crate::requests::user::update_user_data::update_user_data)
//! - Request: [`ApiReqUserUpdateData`](crate::requests::user::update_user_data::ApiReqUserUpdateData)
//! - Response: [`ApiResUserUpdateData`](crate::requests::user::update_user_data::ApiResUserUpdateData)
//!

use std::convert::Infallible;

use postgres::Row as data_row;
use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use hyper::Body;
use hyper::Response;
use hyper::HeaderMap;
use hyper::header::HeaderValue;

use serde::Serialize;
use serde::Deserialize;

use crate::core::core_config::CoreConfig;

use crate::requests::auth::validate_user_token::validate_user_token;
use crate::requests::models::user_data::ModelUserData;

/// ApiReqUserUpdateData
///
/// # Request Type For update_user_data
///
/// Handles updating a `users_data` record in the db
///
/// This type is the deserialized input for:
/// [`update_user_data`](crate::requests::user::update_user_data::update_user_data]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// [`update_user_data`](crate::requests::user::update_user_data::update_user_data)
/// function.
///
/// # Arguments
///
/// * `user_id` - `i32` - user id
/// * `data_id` - `i32` - `users_data.id` record to update
/// * `filename` - `Option<String>` - change the
///   `users_data.filename` field
/// * `data_type` - `Option<String>` - change the
///   `users_data.data_type` field
/// * `comments` - `Option<String>` - change the
///   `users_data.comments` field
/// * `encoding` - `Option<String>` - change the
///   `users_data.encoding` field
/// * `sloc` - `Option<String>` - change the
///   `users_data.sloc` field
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserUpdateData {
    pub user_id: i32,
    pub data_id: i32,
    pub filename: Option<String>,
    pub data_type: Option<String>,
    pub comments: Option<String>,
    pub encoding: Option<String>,
    pub sloc: Option<String>
}

/// implementation for wrapping complex sql statement creation
impl ApiReqUserUpdateData {

    /// get_sql
    ///
    /// Build the update sql statement based off the
    /// object's values
    ///
    pub fn get_sql(
            &self) -> String {
        let mut update_value: String = format!("\
            UPDATE \
                users_data \
            SET ");
        let mut num_params = 0;
        match &self.filename {
            Some(v) => {
                match num_params {
                    0 => {
                        update_value = format!("\
                            {update_value} filename = '{v}'")
                    },
                    _ => {
                        update_value = format!("\
                            {update_value}, filename = '{v}'")
                    }
                }
                num_params += 1;
                0
            },
            None => {
                1
            }
        };
        match &self.data_type {
            Some(v) => {
                match num_params {
                    0 => {
                        update_value = format!("\
                            {update_value} data_type = '{v}'")
                    },
                    _ => {
                        update_value = format!("\
                            {update_value}, data_type = '{v}'")
                    }
                }
                num_params += 1;
                0
            },
            None => {
                1
            }
        };
        match &self.comments {
            Some(v) => {
                match num_params {
                    0 => {
                        update_value = format!("\
                            {update_value} comments = '{v}'")
                    },
                    _ => {
                        update_value = format!("\
                            {update_value}, comments = '{v}'")
                    }
                }
                num_params += 1;
                0
            },
            None => {
                1
            }
        };
        match &self.encoding {
            Some(v) => {
                match num_params {
                    0 => {
                        update_value = format!("\
                            {update_value} encoding = '{v}'")
                    },
                    _ => {
                        update_value = format!("\
                            {update_value}, encoding = '{v}'")
                    }
                }
                num_params += 1;
                0
            },
            None => {
                1
            }
        };
        if false {
            println!("\
                ApiReqUserUpdateData \
                num_params={num_params}");
        }
        // info!("ApiReqUserUpdateData query: {cur_query}");
        return String::from(format!("\
                {} \
                WHERE \
                    users_data.id = {} \
                RETURNING \
                    users_data.id, \
                    users_data.user_id, \
                    users_data.filename, \
                    users_data.data_type, \
                    users_data.size_in_bytes, \
                    users_data.comments, \
                    users_data.encoding, \
                    users_data.sloc, \
                    users_data.created_at, \
                    users_data.updated_at",
                update_value,
                self.data_id));
    }
}

/// ApiResUserUpdateData
///
/// # Response type for update_user_data
///
/// Return user's db record
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`update_user_data`](crate::requests::user::update_user_data::update_user_data]
/// and contained within the
/// hyper [`Body`](hyper::Body)
/// of the
/// hyper [`Response`](hyper::Response)
/// sent back to the client.
///
/// # Arguments
///
/// * `data` - [`ModelUserData`](crate::requests::models::user_data::ModelUserData) -
///   the newly-updated record from the `users_data` db table
/// * `msg` - `String` - help message
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResUserUpdateData {
    pub data: ModelUserData,
    pub msg: String,
}

/// update_user_data
///
/// Handles updating a user data record (in the `users_data` table)
/// based off values in the POST-ed hyper
/// [`Request`](hyper::Request)'s [`Body`](hyper::Body)
///
/// ## Overview Notes
///
/// This function only updates 1 `users_data` record at a time.
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
/// * `bytes` - `&[u8]` - received bytes from the hyper
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
///
/// # Returns
///
/// ## update_user_data on Success Returns
///
/// The newly-updated `users_data` record from the db
/// ([`ModelUserData`](crate::requests::models::user_data::ModelUserData))
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserUpdateData`](crate::requests::user::update_user_data::ApiResUserUpdateData)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `200` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## update_user_data on Failure Returns
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserUpdateData`](crate::requests::user::update_user_data::ApiResUserUpdateData)
/// dictionary with a
/// `non-200` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn update_user_data(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    headers: &HeaderMap<HeaderValue>,
    bytes: &[u8])
-> std::result::Result<Response<Body>, Infallible>
{
    let user_object: ApiReqUserUpdateData = match serde_json::from_slice(&bytes) {
        Ok(uo) => uo,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserUpdateData {
                            data: ModelUserData::default(),
                            msg: format!("\
                                User update data failed - please ensure \
                                user_id and id are set \
                                with optional arguments \
                                filename, size_in_bytes, \
                                comments, data_type, encoding \
                                were set correctly in the request"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };
    let user_id = user_object.user_id;
    let conn = db_pool.get().await.unwrap();
    let _token = match validate_user_token(
            tracking_label,
            &config,
            &conn,
            headers,
            user_object.user_id).await {
        Ok(_token) => _token,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserUpdateData {
                            data: ModelUserData::default(),
                            msg: format!("\
                                User update data failed due to invalid token"),
                        }
                    ).unwrap()))
            .unwrap();
            return Ok(response);
        }
    };

    let cur_query = user_object.get_sql();
    if false {
        println!("\
            {tracking_label} - \
            user update data user_id={user_id} \
            query=\"{cur_query}\"");
    }

    let stmt = conn.prepare(&cur_query).await.unwrap();
    let mut query_result: Vec<data_row> = Vec::with_capacity(100);
    if false { println!("{}", query_result.len()); }
    query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let err_msg = format!("{e}");
            let response = Response::builder()
                .status(500)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserUpdateData {
                            data: ModelUserData::default(),
                            msg: format!("User update data failed for user_id={user_id} with err='{err_msg}'")
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };
    let mut row_list: Vec<ModelUserData> = Vec::with_capacity(1);
    for row in query_result.iter() {
        let found_data_id: i32 = row.try_get("id").unwrap();
        let found_user_id: i32 = row.try_get("user_id").unwrap();
        let found_filename: String = row.try_get("filename").unwrap();
        let found_data_type: String = row.try_get("data_type").unwrap();
        let found_size_in_bytes: i64 = row.try_get("size_in_bytes").unwrap();
        let found_comments: String = row.try_get("comments").unwrap();
        let found_encoding: String = row.try_get("encoding").unwrap();
        let found_sloc: String = row.try_get("sloc").unwrap();
        let created_at_utc: chrono::DateTime<chrono::Utc> = row.try_get("created_at").unwrap();
        let updated_at_str: String = match row.try_get("updated_at") {
            Ok(v) => {
                let updated_at_utc: chrono::DateTime<chrono::Utc> = v;
                format!("{}", updated_at_utc.format("%Y-%m-%dT%H:%M:%SZ"))
            },
            Err(_) => {
                format!("")
            }
        };
        row_list.push(ModelUserData {
            user_id: found_user_id,
            data_id: found_data_id,
            filename: found_filename,
            data_type: found_data_type,
            size_in_bytes: found_size_in_bytes,
            comments: found_comments,
            encoding: found_encoding,
            sloc: found_sloc,
            created_at: format!("{}", created_at_utc.format("%Y-%m-%dT%H:%M:%SZ")),
            updated_at: updated_at_str,
            msg: format!("success"),
        });
    }
    if row_list.len() == 0 {
        let response = Response::builder()
            .status(200)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserUpdateData {
                        data: ModelUserData::default(),
                        msg: format!("no update data found"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
    else {
        let response = Response::builder()
            .status(200)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserUpdateData {
                        data: row_list.remove(0),
                        msg: format!("success"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
}
