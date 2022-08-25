//! ## Consume a One-Time-Use Password Reset Token (OTP)
//!
//! Consume a one-time-use password and change the user's ``users.password`` value to the new argon2-salted password
//!
//! - URL path: ``/user/password/change``
//! - Method: ``POST``
//! - Handler: [`consume_user_otp`](crate::requests::user::consume_user_otp::consume_user_otp)
//! - Request: [`ApiReqUserConsumeOtp`](crate::requests::user::consume_user_otp::ApiReqUserConsumeOtp)
//! - Response: [`ApiResUserConsumeOtp`](crate::requests::user::consume_user_otp::ApiResUserConsumeOtp)
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

use crate::core::core_config::CoreConfig;

use crate::requests::auth::validate_user_token::validate_user_token;
use crate::requests::models::user::get_user_by_id;
use crate::requests::models::user_otp::get_user_otp;

/// ApiReqUserConsumeOtp
///
/// # Request Type For consumer_user_otp
///
/// Handles consuming a user's one-time-use password token in
/// order to reset their password (assuming it is not expired).
///
/// This type is the deserialized input for:
/// [`consume_user_otp`](crate::requests::user::consume_user_otp::consume_user_otp]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// [`consume_user_otp`](crate::requests::user::consume_user_otp::consume_user_otp]
/// function.
///
/// # Arguments
///
/// * `user_id` - `i32` - user id
/// * `email` - `String` - user email
/// * `token` - `String` - user one-time-use token
///   for reseting a user's password
/// * `password` - `String` - new user password
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserConsumeOtp {
    // users.id
    pub user_id: i32,
    // users.email
    pub email: String,
    // users_otp.token
    pub token: String,
    // users.password
    pub password: String,
}

/// ApiResUserConsumeOtp
///
/// # Response type for consumer_user_otp
///
/// Notify the client that:
/// the user's one-time-use password reset token
/// was consumed
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`consume_user_otp`](crate::requests::user::consume_user_otp::consume_user_otp]
/// and contained within the
/// hyper [`Body`](hyper::Body)
/// of the
/// hyper [`Response`](hyper::Response)
/// sent back to the client.
///
/// # Arguments
///
/// * `user_id` - `i32` - user id
/// * `otp_id` - `i32` - users_otp primary db key id
/// * `msg` - `String` - help message
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResUserConsumeOtp {
    // users.id
    pub user_id: i32,
    // users_otp.id
    pub otp_id: i32,
    pub msg: String,
}

/// consume_user_otp
///
/// Handles a user reseting their own password
/// with a one-time-use password token (`otp`).
///
/// ## Overview Notes
///
/// A user can only have one record in the `users_otp` table.
///
/// New password is salted using `argon2`
///
/// OTP tokens can only be used 1 time by a user.
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
/// ## consume_user_otp on Success Returns
///
/// Creates a user in the db and generates a jwt
/// for auth
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserConsumeOtp`](crate::requests::user::consume_user_otp::ApiReqUserConsumeOtp)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `200` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## consume_user_otp on Failure Returns
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserConsumeOtp`](crate::requests::user::consume_user_otp::ApiResUserConsumeOtp)
/// dictionary with a
/// `non-200` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn consume_user_otp(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    headers: &HeaderMap<HeaderValue>,
    bytes: &[u8],
) -> std::result::Result<Response<Body>, Infallible> {
    let req_object: ApiReqUserConsumeOtp = match serde_json::from_slice(bytes) {
        Ok(uo) => uo,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserConsumeOtp {
                        user_id: -1,
                        otp_id: -1,
                        msg: ("User consume one-time-password failed - \
                            please ensure \
                            user_id, email, token, and password \
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
    if req_object.user_id < 0 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id: req_object.user_id,
                    otp_id: -1,
                    msg: ("User consume one-time-password failed \
                        please ensure the \
                        user_id is a non-negative number")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    } else if req_object.email.is_empty() {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id: req_object.user_id,
                    otp_id: -1,
                    msg: ("User consume one-time-password failed \
                        please ensure the \
                        email is set to the user's email address")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    } else if req_object.password.is_empty() {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id: req_object.user_id,
                    otp_id: -1,
                    msg: ("User consume one-time-password failed \
                        please ensure the \
                        passsword is set")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    } else if req_object.password.len() < 4 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id: req_object.user_id,
                    otp_id: -1,
                    msg: ("User consume one-time-password failed \
                        please ensure the \
                        passsword is longer than 4 characters")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    } else if req_object.token.len() < 4 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id: req_object.user_id,
                    otp_id: -1,
                    msg: ("User consume one-time-password failed \
                        please ensure the \
                        token is longer than 4 characters")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    } else if req_object.token.len() > 256 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id: req_object.user_id,
                    otp_id: -1,
                    msg: ("User consume one-time-password failed \
                        please ensure the \
                        token is shorter than 256 characters")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    }

    let conn = db_pool.get().await.unwrap();

    let user_clone = req_object.clone();
    let user_id = user_clone.user_id;
    let user_email = user_clone.email;
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
                    serde_json::to_string(&ApiResUserConsumeOtp {
                        user_id: req_object.user_id,
                        otp_id: -1,
                        msg: ("User consume one-time-password failed \
                            due to invalid token")
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
                "{tracking_label} - \
                failed to consume one-time-password user {user_id} \
                with err='{err_msg}'"
            );
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserConsumeOtp {
                        user_id: req_object.user_id,
                        otp_id: -1,
                        msg: format!(
                            "User consume one-time-password failed - \
                            unable to find user with id: {user_id}"
                        ),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    if user_model.email != req_object.email && user_email != user_model.email {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id: req_object.user_id,
                    otp_id: -1,
                    msg: format!(
                        "User consume one-time-password failed - \
                        user_email does not match {}",
                        req_object.email
                    ),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    }

    // get the user one-time-password record
    let user_otp_model = match get_user_otp(
        tracking_label,
        user_id,
        &req_object.email,
        &req_object.token,
        &conn,
    )
    .await
    {
        Ok(rec) => rec,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserConsumeOtp {
                        user_id: req_object.user_id,
                        otp_id: -1,
                        msg: ("User one-time-password record does not exist")
                            .to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    if req_object.token != user_otp_model.token {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id: req_object.user_id,
                    otp_id: -1,
                    msg: format!(
                        "User one-time-password token={} does not match \
                        db otp_token={}",
                        req_object.token, user_otp_model.token
                    ),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    }

    let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let exp_vs_now_diff =
        now.signed_duration_since(user_otp_model.exp_date_utc);
    let exp_date_vs_now = exp_vs_now_diff.num_seconds();

    // check if the token is expired
    // now - exp_date > 0 == expired
    if exp_date_vs_now > 0 {
        let err_msg = format!(
            "{tracking_label} - user {user_id} \
            one-time-password token {} \
            expired on: \
            exp_date={} \
            duration_since={exp_date_vs_now}s",
            req_object.token, user_otp_model.exp_date_utc
        );
        error!("{err_msg}");
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id: req_object.user_id,
                    otp_id: -1,
                    msg: ("User one-time-password has expired").to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    }

    info!(
        "{tracking_label} - \
        consuming user {user_id} otp"
    );

    let cur_query = format!(
        "UPDATE \
            users_otp \
        SET \
            state = 1, \
            consumed_date = '{now}' \
        WHERE \
            user_id = {user_id} \
            AND \
            state = 0 \
            AND \
            token = '{}' \
            AND \
            email = '{user_email}' \
        RETURNING \
            users_otp.id, \
            users_otp.user_id, \
            users_otp.token, \
            users_otp.email, \
            users_otp.state, \
            users_otp.exp_date;",
        req_object.token
    );

    let stmt = conn.prepare(&cur_query).await.unwrap();
    let query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserConsumeOtp {
                        user_id: req_object.user_id,
                        otp_id: -1,
                        msg: format!(
                            "User consume one-time-password failed \
                            for user_id={user_id} {user_email} \
                            with err='{e}'"
                        ),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
    };

    // must match up with RETURNING
    if let Some(row) = query_result.first() {
        // salt the user's password
        let argon_config = argon_config::default();
        let new_password = argon_hash_encoded(
            req_object.password.as_bytes(),
            &config.server_password_salt,
            &argon_config,
        )
        .unwrap();

        let update_user_query = format!(
            "UPDATE \
                users \
            SET \
                password = '{new_password}' \
            WHERE \
                users.id = {user_id};"
        );
        let stmt = conn.prepare(&update_user_query).await.unwrap();
        let _ = match conn.query(&stmt, &[]).await {
            Ok(query_result) => query_result,
            Err(e) => {
                let response = Response::builder()
                    .status(400)
                    .body(Body::from(
                        serde_json::to_string(&ApiResUserConsumeOtp {
                            user_id: req_object.user_id,
                            otp_id: -1,
                            msg: format!(
                                "User consume one-time-password failed \
                                to reset user's password for \
                                user_id={user_id} {user_email} \
                                with err='{e}'"
                            ),
                        })
                        .unwrap(),
                    ))
                    .unwrap();
                return Ok(response);
            }
        };

        let user_otp_id: i32 = row.try_get("id").unwrap();
        let response = Response::builder()
            .status(200)
            .body(Body::from(
                serde_json::to_string(&ApiResUserConsumeOtp {
                    user_id,
                    otp_id: user_otp_id,
                    msg: "success".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    }
    let response = Response::builder()
        .status(400)
        .body(Body::from(
            serde_json::to_string(&ApiResUserConsumeOtp {
                user_id: req_object.user_id,
                otp_id: -1,
                msg: ("User consume one-time-password failed - \
                    no records found in db")
                    .to_string(),
            })
            .unwrap(),
        ))
        .unwrap();
    Ok(response)
}
