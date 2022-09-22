//! Module for searching for a user's s3 data
//!
//! ## Search for existing user data files from the db
//!
//! Search for matching records in the ``users_data`` db based off the request's values
//!
//! - URL path: ``/user/data/search``
//! - Method: ``POST``
//! - Handler: [`search_user_data`](crate::requests::user::search_user_data::search_user_data)
//! - Request: [`ApiReqUserSearchData`](crate::requests::user::search_user_data::ApiReqUserSearchData)
//! - Response: [`ApiResUserSearchData`](crate::requests::user::search_user_data::ApiResUserSearchData)
//!
use std::convert::Infallible;

use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use hyper::header::HeaderValue;
use hyper::Body;
use hyper::HeaderMap;
use hyper::Response;

use serde::Deserialize;
use serde::Serialize;

use kafka_threadpool::kafka_publisher::KafkaPublisher;

use crate::core::core_config::CoreConfig;
use crate::kafka::publish_msg::publish_msg;
use crate::requests::auth::validate_user_token::validate_user_token;
use crate::requests::models::user_data::ModelUserData;

/// ApiReqUserSearchData
///
/// # Request Type For search_user_data
///
/// Handles searching for many `users_data`
/// record(s) from the db with optional filters
///
/// This type is the deserialized input for:
/// [`search_user_data`](crate::requests::user::search_user_data::search_user_data]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// [`search_user_data`](crate::requests::user::search_user_data::search_user_data)
/// function.
///
/// # Arguments
///
/// * `user_id` - `i32` - user id
/// * `creator_user_id` - `Option<i32>` - filter by
///   `users_data.user_id`
/// * `data_id` - `Option<i32>` - filter by
///   `users_data.id`
/// * `filename` - `Option<String>` - filter by
///   `users_data.filename` with a ILIKE operation
/// * `data_type` - `Option<String>` - filter by
///   `users_data.data_type`
///   <https://www.google.com/search?q=rust+bigint+postgres>
///   postgres size_in_bytes field is a BIGINT type
/// * `above_bytes` - `Option<i64>` - filter by
///   `users_data.above_bytes`
///   (relative to `users_data.size_in_bytes` value)
/// * `below_bytes` - `Option<i64>` - filter by
///   `users_data.below_bytes`
///   (relative to `users_data.size_in_bytes` value)
/// * `comments` - `Option<String>` - filter by
///   `users_data.comments` with a ILIKE operation
/// * `encoding` - `Option<String>` - filter by
///   `users_data.encoding`
/// * `sloc` - `Option<String>` - filter by
///   `users_data.sloc` the s3 storage location
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserSearchData {
    pub user_id: i32,
    pub creator_user_id: Option<i32>,
    pub data_id: Option<i32>,
    pub filename: Option<String>,
    pub data_type: Option<String>,
    // https://www.google.com/search?q=rust+bigint+postgres
    // postgres size_in_bytes field is a BIGINT type
    pub above_bytes: Option<i64>,
    pub below_bytes: Option<i64>,
    pub comments: Option<String>,
    pub encoding: Option<String>,
    pub sloc: Option<String>,
}

/// implementation for handling complex search filtering
/// using sql
impl ApiReqUserSearchData {
    /// get_sql
    ///
    /// Build the v1 search query string based on the
    /// the requested values.
    ///
    pub fn get_sql(&self) -> String {
        let mut update_value: String = format!(
            "SELECT \
                users_data.id, \
                users_data.user_id, \
                users_data.filename, \
                users_data.size_in_bytes, \
                users_data.comments, \
                users_data.data_type, \
                users_data.encoding, \
                users_data.sloc, \
                users_data.created_at, \
                users_data.updated_at \
            FROM \
                users_data \
            WHERE \
                users_data.user_id = {}",
            self.user_id
        );
        match self.creator_user_id {
            Some(_) => {
                match update_value.len() {
                    0 => {
                        // update_value = format!("{update_value}, {creator_user_id_value}");
                        0
                    }
                    // only one user_id supported for now
                    _ => 1,
                }
            }
            None => 1,
        };
        match self.data_id {
            Some(v) => {
                update_value = format!("{update_value}, id = {}", v);
                0
            }
            None => 1,
        };
        match &self.filename {
            Some(v) => {
                update_value =
                    format!("{update_value}, filename ILIKE '%{v}%'");
                0
            }
            None => 1,
        };
        match &self.data_type {
            Some(v) => {
                update_value =
                    format!("{update_value}, data_type ILIKE '%{v}%'");
                0
            }
            None => 1,
        };
        // https://www.google.com/search?q=rust+bigint+postgres
        // postgres size_in_bytes field is a BIGINT type
        match self.above_bytes {
            Some(v) => {
                update_value = format!("{update_value}, size_in_bytes > {v}");
                0
            }
            None => 1,
        };
        match self.below_bytes {
            Some(v) => {
                update_value = format!("{update_value}, size_in_bytes < {v}");
                0
            }
            None => 1,
        };
        match &self.comments {
            Some(v) => {
                update_value =
                    format!("{update_value}, comments ILIKE '%{v}%'");
                0
            }
            None => 1,
        };
        match &self.encoding {
            Some(v) => {
                update_value =
                    format!("{update_value}, encoding ILIKE '%{v}%'");
                0
            }
            None => 1,
        };
        match &self.sloc {
            Some(v) => {
                update_value = format!("{update_value}, sloc ILIKE '%{v}%'");
                0
            }
            None => 1,
        };
        // info!("ApiReqUserSearchData query: {cur_query}");
        format!(
            "{} ORDER BY users_data.id DESC \
                LIMIT 100;",
            update_value
        )
    }
}

/// ApiResUserSearchData
///
/// # Response type for search_user_data
///
/// Contains matching `users_data` records from the
/// db based off the POST-ed filters in the type:
/// [`ApiReqUserSearchData`](crate::requests::user::search_user_data::ApiReqUserSearchData)
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`search_user_data`](crate::requests::user::search_user_data::search_user_data]
/// and contained within the
/// hyper [`Body`](hyper::Body)
/// of the
/// hyper [`Response`](hyper::Response)
/// sent back to the client.
///
/// # Arguments
///
/// * `data` - Vec<[`ModelUserData`](crate::requests::models::user_data::ModelUserData)> -
///   list of matching `users_data` records
/// * `msg` - `String` - help message
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResUserSearchData {
    pub data: Vec<ModelUserData>,
    pub msg: String,
}

/// search_user_data
///
/// Search for matching `users_data` records by the POST-ed
/// [`ApiReqUserSearchData`](crate::requests::user::search_user_data::ApiReqUserSearchData)
/// (filters) and return a list of
/// [`ModelUserData`](crate::requests::models::user_data)
/// within the
/// [`ApiResUserSearchData`](crate::requests::user::search_user_data::ApiResUserSearchData)
///
/// ## Overview Notes
///
/// A user can have many records in the `users_data` table.
///
/// # Arguments
///
/// * `tracking_label` - `&str` - caller logging label
/// * `config` - [`CoreConfig`](crate::core::core_config::CoreConfig)
/// * `db_pool` - [`Pool`](bb8::Pool) - postgres client
///   db threadpool with required tls encryption
/// * `kafka_pool` -
///   [`KafkaPublisher`](kafka_threadpool::kafka_publisher::KafkaPublisher)
///   for asynchronously publishing messages to the connected kafka cluster
/// * `headers` - [`HeaderMap`](hyper::HeaderMap) -
///   hashmap containing headers in key-value pairs
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
/// * `bytes` - `&[u8]` - received bytes from the hyper
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
///
/// # Returns
///
/// ## search_user_data on Success Returns
///
/// List of matching `users_data` records from the db
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserSearchData`](crate::requests::user::search_user_data::ApiResUserSearchData)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `200` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## search_user_data on Failure Returns
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserSearchData`](crate::requests::user::search_user_data::ApiResUserSearchData)
/// dictionary with a
/// `non-200` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn search_user_data(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    kafka_pool: &KafkaPublisher,
    headers: &HeaderMap<HeaderValue>,
    bytes: &[u8],
) -> std::result::Result<Response<Body>, Infallible> {
    let user_object: ApiReqUserSearchData = match serde_json::from_slice(bytes)
    {
        Ok(uo) => uo,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserSearchData {
                        data: Vec::new(),
                        msg: ("User search data failed - please ensure \
                            user_id is set \
                            with optional arguments \
                            user_id, creator_user_id, \
                            data_id, filename, data_type, \
                            above_bytes, below_bytes, \
                            comments, encoding, sloc \
                            were set correctly in the request")
                            .to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };
    let user_id = user_object.user_id;
    let conn = db_pool.get().await.unwrap();
    let _token = match validate_user_token(
        tracking_label,
        config,
        &conn,
        headers,
        user_id,
    )
    .await
    {
        Ok(_token) => _token,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserSearchData {
                        data: Vec::new(),
                        msg: ("User search data failed due to invalid token")
                            .to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    let cur_query = user_object.get_sql();
    /*
    if false {
        println!(
            "{tracking_label} - \
            searching for user_id={user_id} data with \
            query {cur_query}"
        );
    }
    */

    let stmt = conn.prepare(&cur_query).await.unwrap();
    let query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let err_msg = format!("{e}");
            let response = Response::builder()
                .status(500)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserSearchData {
                            data: Vec::new(),
                            msg: format!("User data search failed for user_id={user_id} with err='{err_msg}'")
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
        let created_at_utc: chrono::DateTime<chrono::Utc> =
            row.try_get("created_at").unwrap();
        let updated_at_str: String = match row.try_get("updated_at") {
            Ok(v) => {
                let updated_at_utc: chrono::DateTime<chrono::Utc> = v;
                format!("{}", updated_at_utc.format("%Y-%m-%dT%H:%M:%SZ"))
            }
            Err(_) => "".to_string(),
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
            created_at: format!(
                "{}",
                created_at_utc.format("%Y-%m-%dT%H:%M:%SZ")
            ),
            updated_at: updated_at_str,
            msg: "success".to_string(),
        });
    }
    if row_list.is_empty() {
        // if enabled, publish to kafka
        if config.kafka_publish_events {
            publish_msg(
                kafka_pool,
                // topic
                "user.events",
                // partition key
                &format!("user-{}", user_id),
                // optional headers stored in: Option<HashMap<String, String>>
                None,
                // payload in the message
                &format!("SEARCH_USER_DATA user={user_id}"),
            )
            .await;
        }

        let response = Response::builder()
            .status(200)
            .body(Body::from(
                serde_json::to_string(&ApiResUserSearchData {
                    data: Vec::new(),
                    msg: "no search data found".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        Ok(response)
    } else {
        let response = Response::builder()
            .status(200)
            .body(Body::from(
                serde_json::to_string(&ApiResUserSearchData {
                    data: row_list,
                    msg: "success".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        Ok(response)
    }
}
