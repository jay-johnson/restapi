//! Module for deleting a user
//!
//! ## Delete User
//!
//! Delete a single ``users`` record (note: this does not delete the db record, just sets the ``users.state`` to inactive ``1``)
//!
//! - URL path: ``/user``
//! - Method: ``DELETE``
//! - Handler: [`delete_user`](crate::requests::user::delete_user::delete_user)
//! - Request: [`ApiReqUserDelete`](crate::requests::user::delete_user::ApiReqUserDelete)
//! - Response: [`ApiResUserDelete`](crate::requests::user::delete_user::ApiResUserDelete)
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

/// ApiReqUserDelete
///
/// # Request Type For delete_user
///
/// Handles deleting (deactivating) a user
///
/// This type is the deserialized input for:
/// [`delete_user`](crate::requests::user::delete_user::delete_user]
///
/// # Note
///
/// This does not remove the user record, instead it
/// changes the `users.state` from
/// *active* (`0`)
/// to
/// *inactive* (`1`)
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// [`delete_user`](crate::requests::user::delete_user::delete_user)
/// function.
///
/// # Arguments
///
/// * `user_id` - `i32` - user id
/// * `email` - `String` - user email
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserDelete {
    pub user_id: i32,
    pub email: String,
}

/// ApiResUserDelete
///
/// # Response type for delete_user
///
/// Notify the client that:
/// the user's account has been deleted
/// (deactivated for recovery purposes)
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`delete_user`](crate::requests::user::delete_user::delete_user]
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
pub struct ApiResUserDelete {
    pub user_id: i32,
    pub email: String,
    pub state: i32,
    pub verified: i32,
    pub role: String,
    pub msg: String,
}

/// delete_user
///
/// Handles deleting a user by changing the
/// `users.state` to `1`.
/// This change is enforced on
/// [`login_user`](crate::requests::auth::login_user::login_user)
/// when the user tries to login again and on the
/// [`validate_user_token`](crate::requests::auth::validate_user_token::validate_user_token)
/// for any existing user jwt's.
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
/// * `bytes` - `&[u8]` - received bytes from the hyper
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
///
/// # Returns
///
/// ## Success
///
/// Deletes a user in the db
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserDelete`](crate::requests::user::delete_user::ApiResUserDelete)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `204` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserDelete`](crate::requests::user::delete_user::ApiResUserDelete)
/// dictionary with a
/// `non-204` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn delete_user(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    kafka_pool: &KafkaPublisher,
    headers: &HeaderMap<HeaderValue>,
    bytes: &[u8],
) -> std::result::Result<Response<Body>, Infallible> {
    let user_object: ApiReqUserDelete = match serde_json::from_slice(bytes) {
        Ok(uo) => uo,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserDelete {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: ("User delete failed - please ensure user_id \
                                and user_email were set on the request")
                            .to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    let conn = db_pool.get().await.unwrap();
    let _token = match validate_user_token(
        tracking_label,
        config,
        &conn,
        headers,
        user_object.user_id,
    )
    .await
    {
        Ok(_token) => _token,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserDelete {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: ("User delete failed due to invalid token")
                            .to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    let query = format!(
        "UPDATE \
            users \
        SET \
            state = 1 \
        WHERE \
            email = {} \
        RETURNING \
            users.id, \
            users.email, \
            users.state, \
            users.verified, \
            users.role;",
        user_object.email
    );
    let stmt = conn.prepare(&query).await.unwrap();
    let query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let err_msg = format!("{}", e);
            let response = Response::builder()
                .status(500)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserDelete {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: format!(
                            "User delete failed for email={} \
                            with err='{err_msg}'",
                            user_object.email
                        ),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };
    let mut row_list: Vec<(i32, String, i32, i32, String)> =
        Vec::with_capacity(1);
    if let Some(row) = query_result.first() {
        let id: i32 = row.try_get("id").unwrap();
        let email: String = row.try_get("email").unwrap();
        let user_state: i32 = row.try_get("state").unwrap();
        let user_verified: i32 = row.try_get("verified").unwrap();
        let role: String = row.try_get("role").unwrap();
        row_list.push((id, email, user_state, user_verified, role))
    }
    if row_list.is_empty() {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserDelete {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: format!(
                            "User creation failed - unable to find user by email={}",
                                user_object.email)
                    }
                ).unwrap()))
            .unwrap();
        Ok(response)
    } else {
        // if enabled, publish to kafka
        if config.kafka_publish_events {
            publish_msg(
                kafka_pool,
                // topic
                "user.events",
                // partition key
                &format!("user-{}", user_object.user_id),
                // optional headers stored in: Option<HashMap<String, String>>
                None,
                // payload in the message
                &format!("USER_DELETE user={}", user_object.user_id),
            )
            .await;
        }

        let response = Response::builder()
            .status(204)
            .body(Body::from(
                serde_json::to_string(&ApiResUserDelete {
                    user_id: row_list[0].0,
                    email: row_list[0].1.clone(),
                    state: row_list[0].2,
                    verified: row_list[0].3,
                    role: row_list[0].4.clone(),
                    msg: "success".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        Ok(response)
    }
}
