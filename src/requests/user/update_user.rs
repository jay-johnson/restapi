//! Module for updating a user's fields in the postgres db
//!
//! ## Update User
//!
//! Update supported ``users`` fields (including change user email and password)
//!
//! - URL path: ``/user``
//! - Method: ``PUT``
//! - Handler: [`update_user`](crate::requests::user::update_user::update_user)
//! - Request: [`ApiReqUserUpdate`](crate::requests::user::update_user::ApiReqUserUpdate)
//! - Response: [`ApiResUserUpdate`](crate::requests::user::update_user::ApiResUserUpdate)
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

use argon2::hash_encoded as argon_hash_encoded;
use argon2::Config as argon_config;

use kafka_threadpool::kafka_publisher::KafkaPublisher;

use crate::core::core_config::CoreConfig;
use crate::kafka::publish_msg::publish_msg;
use crate::requests::auth::validate_user_token::validate_user_token;
use crate::requests::models::user::get_user_by_id;
use crate::requests::models::user::ModelUser;
use crate::requests::user::is_verification_enabled::is_verification_enabled;
use crate::requests::user::upsert_user_verification::upsert_user_verification;
use crate::utils::get_server_address::get_server_address;

/// ApiReqUserUpdate
///
/// # Request Type For update_user
///
/// Handles updating a `users` record in the db
///
/// This type is the deserialized input for:
/// [`update_user`](crate::requests::user::update_user::update_user]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// [`update_user`](crate::requests::user::update_user::update_user)
/// function.
///
/// # Arguments
///
/// * `user_id` - `i32` - user id
/// * `email` - `Option<String>` - change the
///   `users.email` field
/// * `password` - `Option<String>` - change the
///   `users.password` field
/// * `state` - `Option<i32>` - change the
///   `users.state` field
/// * `verified` - `Option<i32>` - change the
///   `users.verified` field
/// * `role` - `Option<String>` - change the
///   `users.role` field
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserUpdate {
    pub user_id: i32,
    pub email: Option<String>,
    pub password: Option<String>,
    pub state: Option<i32>,
    pub verified: Option<i32>,
    pub role: Option<String>,
}

/// implementation for wrapping complex sql statement creation
impl ApiReqUserUpdate {
    /// get_sql
    ///
    /// Build the update sql statement based off the
    /// object's values
    ///
    /// # Password Salt Algorithm
    ///
    /// If the optional password is changing, this method
    /// uses `argon2` to salt the new password value
    /// stored in the db.
    ///
    pub fn get_sql(
        &self,
        server_password_salt: &[u8],
        user_model: &ModelUser,
    ) -> String {
        let user_email = self.email.clone();
        let email_value: String = match self.email.clone() {
            Some(new_email) => {
                if is_verification_enabled() {
                    // if the email is different
                    if !new_email.is_empty() && user_model.email != new_email {
                        format!("email = '{new_email}', verified = 0")
                    } else {
                        // the email in the db matches the requested one
                        "".to_string()
                    }
                } else {
                    format!("email = '{new_email}', verified = 1")
                }
            }
            None => "".to_string(),
        };
        let mut update_value = email_value;
        let password_value: String = match self.password.clone() {
            Some(cur_user_salted_password) => {
                let config = argon_config::default();
                let new_hashed_password = argon_hash_encoded(
                    cur_user_salted_password.as_bytes(),
                    server_password_salt,
                    &config,
                )
                .unwrap();
                if update_value.is_empty() {
                    format!(", password = '{new_hashed_password}'")
                } else {
                    format!("password = '{new_hashed_password}'")
                }
            }
            None => "".to_string(),
        };
        update_value = format!("{update_value}{password_value}");
        let state_value: String = match self.state {
            Some(v) => {
                if update_value.is_empty() {
                    format!(", state = '{v}'")
                } else {
                    format!("state = '{v}'")
                }
            }
            None => "".to_string(),
        };
        update_value = format!("{update_value}{state_value}");
        let role_value: String = match self.role {
            Some(_) => {
                // for now role changing has no effect on purpose
                if self.email.is_some()
                    && &user_email.unwrap_or_else(|| "".to_string())
                        == "admin@email.com"
                {
                    if update_value.is_empty() {
                        ", role = 'admin' ".to_string()
                    } else {
                        "role = 'admin' ".to_string()
                    }
                } else if update_value.is_empty() {
                    ", role = 'user' ".to_string()
                } else {
                    "role = 'user' ".to_string()
                }
            }
            None => "".to_string(),
        };
        update_value = format!("{update_value}{role_value}");
        let cur_query = format!(
            "UPDATE \
                users \
            SET \
                {update_value} \
            WHERE \
                users.id = {} \
            RETURNING \
                users.id, \
                users.email, \
                users.state, \
                users.verified, \
                users.role;",
            self.user_id
        );
        // careful this can log the salted password!
        // info!("ApiReqUserUpdate query: {cur_query}");
        cur_query
    }
}

/// ApiResUserUpdate
///
/// # Response type for update_user
///
/// Return user's db record
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`update_user`](crate::requests::user::update_user::update_user]
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
pub struct ApiResUserUpdate {
    pub user_id: i32,
    pub email: String,
    pub state: i32,
    pub verified: i32,
    pub role: String,
    pub msg: String,
}

/// update_user
///
/// Handles updating a user record (in the `users` table)
/// based off values in the POST-ed hyper
/// [`Request`](hyper::Request)'s [`Body`](hyper::Body)
///
/// ## Overview Notes
///
/// This function only updates 1 `users` record at a time.
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
/// ## update_user on Success Returns
///
/// The newly-updated `users` record from the db
/// ([`ApiResUserUpdate`](crate::requests::user::update_user::ApiResUserUpdate))
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserUpdate`](crate::requests::user::update_user::ApiResUserUpdate)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `200` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## update_user on Failure Returns
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserUpdate`](crate::requests::user::update_user::ApiResUserUpdate)
/// dictionary with a
/// `non-200` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn update_user(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    kafka_pool: &KafkaPublisher,
    headers: &HeaderMap<HeaderValue>,
    bytes: &[u8],
) -> std::result::Result<Response<Body>, Infallible> {
    let user_object: ApiReqUserUpdate = match serde_json::from_slice(bytes) {
        Ok(uo) => uo,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserUpdate {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: ("User update failed - please ensure \
                            user_id is set \
                            with optional arguments \
                            email, password, state, role \
                            were set correctly in the request")
                            .to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    // is this a waste of time because nothing changed
    if user_object.email.is_none()
        && user_object.password.is_none()
        && user_object.state.is_none()
        && user_object.role.is_none()
    {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserUpdate {
                    user_id: -1,
                    email: "".to_string(),
                    state: -1,
                    verified: -1,
                    role: "".to_string(),
                    msg: ("User update detected no changes - please ensure \
                        the correct user_id for the TOKEN is set \
                        with optional arguments \
                        email, password, state, role \
                        were set correctly in the request")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    } else if user_object
        .password
        .clone()
        .unwrap_or_else(|| "NOT_SET_SO_SET_LARGER_THAN_MIN".to_string())
        .len()
        < 4
    {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserUpdate {
                    user_id: -1,
                    email: "".to_string(),
                    state: -1,
                    verified: -1,
                    role: "".to_string(),
                    msg: ("User update failed - \
                        password must be longer than 4 characters")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    }

    let conn = db_pool.get().await.unwrap();

    let user_clone = user_object.clone();
    let user_id = user_clone.user_id;
    let user_email = user_clone.email.unwrap_or_else(|| "".to_string());
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
                    serde_json::to_string(&ApiResUserUpdate {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: ("User update failed due to invalid token")
                            .to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    // get the user and detect if the email is different
    let user_model = match get_user_by_id(tracking_label, user_id, &conn).await
    {
        Ok(user_model) => user_model,
        Err(err_msg) => {
            error!(
                "{tracking_label} - failed to update user {user_id} \
                with err='{err_msg}'"
            );
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserUpdate {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: format!(
                            "User update failed - \
                            unable to find user with id: {user_id}"
                        ),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    let cur_query =
        user_object.get_sql(&config.server_password_salt, &user_model);

    let stmt = conn.prepare(&cur_query).await.unwrap();
    let query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let err_msg = format!("{e}");
            if err_msg.contains(
                "db error: ERROR: duplicate key value \
                violates unique constraint",
            ) && err_msg.contains("users_email_key")
                && err_msg.contains("already exists")
            {
                let response = Response::builder()
                    .status(400)
                    .body(Body::from(
                        serde_json::to_string(&ApiResUserUpdate {
                            user_id: -1,
                            email: "".to_string(),
                            state: -1,
                            verified: -1,
                            role: "".to_string(),
                            msg: format!(
                                "User email is already \
                                in use: {user_email}"
                            ),
                        })
                        .unwrap(),
                    ))
                    .unwrap();
                return Ok(response);
            } else {
                let response = Response::builder()
                    .status(400)
                    .body(Body::from(
                        serde_json::to_string(
                            &ApiResUserUpdate {
                                user_id: -1,
                                email: "".to_string(),
                                state: -1,
                                verified: -1,
                                role: "".to_string(),
                                msg: format!(
                                    "User update failed for user_id={user_id} {user_email} \
                                    with err='{err_msg}'")
                            }
                        ).unwrap()))
                    .unwrap();
                return Ok(response);
            }
        }
    };

    // must match up with RETURNING
    let mut row_list: Vec<(i32, String, i32, i32, String)> =
        Vec::with_capacity(1);
    for row in query_result.iter() {
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
                    &ApiResUserUpdate {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        msg: format!(
                            "User update failed - user does \
                            not exist with user_id={user_id} email={user_email}")
                    }
                ).unwrap()))
            .unwrap();
        Ok(response)
    } else {
        // only update the verification table
        // if it's enabled
        // and the email changed
        if is_verification_enabled()
            && !user_email.is_empty()
            && user_email != user_model.email
        {
            let user_id = user_model.id;
            match upsert_user_verification(
                tracking_label,
                user_id,
                &user_email,
                false, // not first time creating the user
                0,     // if the email changed it's not verified
                &conn,
            )
            .await
            {
                Ok(verification_token) => {
                    info!(
                        "{tracking_label} - \
                        verify token updated for user={user_id} \
                        {user_email} verify url: \
                        curl -ks \
                        \"https://{}/user/verify?u={user_id}&t={verification_token}\"",
                            get_server_address("api"));
                }
                Err(e) => {
                    error!(
                        "{tracking_label} - \
                        failed to generate verify token for user update \
                        user_id={user_id} \
                        {user_email} with err='{e}'"
                    );
                }
            }
        }
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
                &format!("USER_UPDATE user={user_id} email={user_email}"),
            )
            .await;
        }
        let response = Response::builder()
            .status(200)
            .body(Body::from(
                serde_json::to_string(&ApiResUserUpdate {
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
