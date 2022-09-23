//! Module for getting a user
//!
//! ## Get User
//!
//! Get a single user by ``users.id`` - by default, a user can only get their own account details
//!
//! - URL path: ``/user/USERID``
//! - Method: ``GET``
//! - Handler: [`get_user`](crate::requests::user::get_user::get_user)
//! - Request: [`ApiReqUserGet`](crate::requests::user::get_user::ApiReqUserGet)
//! - Response: [`ApiResUserGet`](crate::requests::user::get_user::ApiResUserGet)
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
use crate::requests::models::user::get_user_by_id;

/// ApiReqUserGet
///
/// # Request Type For get_user
///
/// Handles getting a user from the db
///
/// This type is the deserialized input for:
/// [`get_user`](crate::requests::user::get_user::get_user]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `request_uri` (`&str`) argument
/// on the
/// [`get_user`](crate::requests::user::get_user::get_user)
/// function.
///
/// # Arguments
///
/// * `user_id` - `i32` - user id
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserGet {
    pub user_id: i32,
}

/// ApiResUserGet
///
/// # Response type for get_user
///
/// Return user's db record
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`get_user`](crate::requests::user::get_user::get_user]
/// and contained within the
/// hyper [`Body`](hyper::Body)
/// of the
/// hyper [`Response`](hyper::Response)
/// sent back to the client.
///
/// # Arguments
///
/// * `user_id` - `i32` - user id
/// * `email` - `String` - user email
/// * `state` - `i32` - user state (`1` - inactive)
/// * `verified` - `i32` - user email verified
///   (`0` - not-verified, `1` - verified)
/// * `role` - `String` - user role
/// * `msg` - `String` - help message
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResUserGet {
    pub user_id: i32,
    pub email: String,
    pub state: i32,
    pub verified: i32,
    pub role: String,
    pub msg: String,
}

/// get_user
///
/// Get the user's own db record
///
/// ## Overview Notes
///
/// A user can only have one record in the `users` table.
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
/// * `request_uri` - `&str` - url on the HTTP request
///   ([`handle_request`](crate::handle_request::handle_request) extracts
///   the url part of the
///   [`Request`](hyper::Request))
///
/// # Returns
///
/// ## get_user on Success Returns
///
/// A single user in the db
/// ([`ApiResUserGet`](crate::requests::user::get_user::ApiResUserGet))
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserGet`](crate::requests::user::get_user::ApiResUserGet)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `200` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## get_user on Failure Returns
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserGet`](crate::requests::user::get_user::ApiResUserGet)
/// dictionary with a
/// `non-200` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn get_user(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    kafka_pool: &KafkaPublisher,
    headers: &HeaderMap<HeaderValue>,
    request_uri: &str,
) -> std::result::Result<Response<Body>, Infallible> {
    let user_id = str::replace(request_uri, "/user/", "")
        .parse::<i32>()
        .unwrap_or(-1);
    if user_id <= 0 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserGet {
                    user_id: -1,
                    email: "".to_string(),
                    state: -1,
                    verified: -1,
                    role: "".to_string(),
                    msg: ("Invalid user_id must be a positive integer")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    }

    info!("{tracking_label} - getting user_id={user_id}");
    let user_object = ApiReqUserGet { user_id };

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
                    serde_json::to_string(&ApiResUserGet {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: ("User get failed due to invalid token")
                            .to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    // find all user by email and an active state where state == 0
    match get_user_by_id(tracking_label, user_id, &conn).await {
        Ok(user_model) => {
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
                    &format!("USER_GET user={user_id}"),
                )
                .await;
            }

            let response = Response::builder()
                .status(200)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserGet {
                        user_id: user_model.id,
                        email: user_model.email,
                        state: user_model.state,
                        verified: user_model.verified,
                        role: user_model.role,
                        msg: "success".to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            Ok(response)
        }
        Err(err_msg) => {
            error!(
                "{tracking_label} - \
                failed to get user by id with err={err_msg}"
            );
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserGet {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: format!(
                            "User login failed - \
                                user does not exist with user_id={}",
                            user_object.user_id
                        ),
                    })
                    .unwrap(),
                ))
                .unwrap();
            Ok(response)
        }
    }
}
