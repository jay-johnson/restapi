//! ## Create One-Time-Use Password Reset Token (OTP)
//!
//! Create a one-time-use password reset token that allows a user to change their ``users.password`` value by presenting the token
//!
//! - URL path: ``/user/password/reset``
//! - Method: ``POST``
//! - Handler: [`create_otp`](crate::requests::user::create_otp::create_otp)
//! - Request: [`ApiReqUserCreateOtp`](crate::requests::user::create_otp::ApiReqUserCreateOtp)
//! - Response: [`ApiResUserCreateOtp`](crate::requests::user::create_otp::ApiResUserCreateOtp)
//!

use std::convert::Infallible;

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
use crate::utils::get_uuid::get_uuid;

use crate::requests::models::user::get_user_by_id;
use crate::requests::auth::validate_user_token::validate_user_token;

/// ApiReqUserCreateOtp
///
/// # Request Type For create_otp
///
/// Creating a one-time-use password token for helping a user reset
/// their password (note: users should have a verified email to avoid
/// email spam).
///
/// This type is the deserialized input for:
/// [`create_otp`](crate::requests::user::create_otp::create_otp]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// `create_otp`
/// function.
///
/// # Fields
///
/// * `user_id` - `i32` - user id
/// * `email` - `String` - user email
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserCreateOtp {
    // users.id
    pub user_id: i32,
    // users.email
    pub email: String,
}

/// ApiResUserCreateOtp
///
/// # Response type for create_otp
///
/// Notify the client that, the user's one-time-use password reset token
/// was consumed, and the user will need to call:
/// [`login_user`](crate::requests::auth::login_user::login_user)
/// to log in with the new password.
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`create_otp`](crate::requests::user::create_otp::create_otp]
/// and contained within the
/// hyper [`Body`](hyper::Body)
/// of the
/// hyper [`Response`](hyper::Response)
/// sent back to the client.
///
/// # Fields
///
/// * `user_id` - `i32` - user id
/// * `token` - `String` - user's new one-time-use token to reset
///   their password
/// * `exp_date` - `String` - UTC-formatted date time string when
///   the `token` expires
/// * `msg` - `String` - help message
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResUserCreateOtp {
    // users.id
    pub user_id: i32,
    // users_otp.token
    pub token: String,
    pub exp_date: String,
    pub msg: String,
}

/// create_otp
///
/// Creates a one-time-use token to reset a user's account password.
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
/// ## Success
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserCreateOtp`](crate::requests::user::create_otp::ApiResUserCreateOtp)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `201` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserCreateOtp`](crate::requests::user::create_otp::ApiResUserCreateOtp)
/// dictionary with a
/// `non-201` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn create_otp(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    headers: &HeaderMap<HeaderValue>,
    bytes: &[u8])
-> std::result::Result<Response<Body>, Infallible>
{
    let req_object: ApiReqUserCreateOtp = match serde_json::from_slice(&bytes) {
        Ok(uo) => uo,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserCreateOtp {
                            user_id: -1,
                            token: format!(""),
                            exp_date: format!(""),
                            msg: format!("\
                                User create one-time-password failed - \
                                please ensure \
                                user_id and email \
                                were set correctly in the request"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };

    // is this a waste of time because nothing changed
    if
            req_object.user_id < 0 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserCreateOtp {
                        user_id: req_object.user_id,
                        token: format!(""),
                        exp_date: format!(""),
                        msg: format!("\
                            User create one-time-password failed \
                            please ensure \
                            user_id is a non-negative number"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
    else if req_object.email == "" {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserCreateOtp {
                        user_id: req_object.user_id,
                        token: format!(""),
                        exp_date: format!(""),
                        msg: format!("\
                            User create one-time-password failed \
                            please ensure \
                            email is set to the user's email address"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    let conn = db_pool.get().await.unwrap();

    let user_clone = req_object.clone();
    let user_id = user_clone.user_id;
    let user_email = user_clone.email;
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
                        &ApiResUserCreateOtp {
                            user_id: req_object.user_id,
                            token: format!(""),
                            exp_date: format!(""),
                            msg: format!("\
                                User create one-time-password failed \
                                due to invalid token"),
                        }
                    ).unwrap()))
            .unwrap();
            return Ok(response);
        }
    };

    // get the user and detect if the email is different
    let user_model = match get_user_by_id(
            tracking_label,
            user_id,
            &conn).await {
        Ok(user_model) => {
            user_model
        },
        Err(err_msg) => {
            error!("\
                {tracking_label} - \
                failed to create one-time-password user {user_id} \
                with err='{err_msg}'");
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserCreateOtp {
                            user_id: req_object.user_id,
                            token: format!(""),
                            exp_date: format!(""),
                            msg: format!("\
                                User create one-time-password failed - \
                                unable to find user with id: {user_id}"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        },
    };

    if user_model.email != req_object.email {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserCreateOtp {
                            user_id: req_object.user_id,
                            token: format!(""),
                            exp_date: format!(""),
                            msg: format!("\
                                User create one-time-password failed - \
                                user_email does not match {}",
                                    req_object.email),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
    }

    let user_otp_expiration_in_seconds_str =
        std::env::var("USER_OTP_EXP_IN_SECONDS")
        .unwrap_or(String::from("2592000"));
    let user_otp_expiration_in_seconds: i64 =
        user_otp_expiration_in_seconds_str.parse::<i64>().unwrap();
    let now = chrono::Utc::now();
    // https://docs.rs/chrono/0.4.19/chrono/struct.Duration.html#method.seconds
    let otp_expiration_timestamp =
        now + chrono::Duration::seconds(user_otp_expiration_in_seconds);

    let otp_token = format!("\
        {}{}",
        get_uuid(),
        get_uuid());

    let cur_query = format!("\
        INSERT INTO \
            users_otp (\
                user_id, \
                token, \
                email, \
                state, \
                exp_date) \
        VALUES (\
            {user_id}, \
            '{otp_token}', \
            '{user_email}', \
            0,
            '{otp_expiration_timestamp}') \
        RETURNING \
            users_otp.id, \
            users_otp.user_id, \
            users_otp.token, \
            users_otp.email, \
            users_otp.state, \
            users_otp.exp_date;");

    let stmt = conn.prepare(&cur_query).await.unwrap();
    let query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserCreateOtp {
                            user_id: req_object.user_id,
                            token: format!(""),
                            exp_date: format!(""),
                            msg: format!("\
                                User create one-time-password failed \
                                for user_id={user_id} {user_email} \
                                with err='{e}'")
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        },
    };

    // must match up with RETURNING
    for row in query_result.iter() {
        let user_otp_id: i32 = row.try_get("id").unwrap();
        let user_otp_token: String = row.try_get("token").unwrap();
        let user_otp_exp_date_str: String = match row.try_get("exp_date") {
            Ok(v) => {
                let user_otp_exp_date: chrono::DateTime<chrono::Utc> = v;
                format!("{}", user_otp_exp_date.format("%Y-%m-%dT%H:%M:%SZ"))
            },
            Err(_) => {
                format!("")
            }
        };
        let response = Response::builder()
            .status(201)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserCreateOtp {
                        user_id: user_otp_id,
                        token: user_otp_token,
                        exp_date: user_otp_exp_date_str,
                        msg: format!("success"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
    let response = Response::builder()
        .status(400)
        .body(Body::from(
            serde_json::to_string(
                &ApiResUserCreateOtp {
                    user_id: req_object.user_id,
                    token: format!(""),
                    exp_date: format!(""),
                    msg: format!("\
                        User create one-time-password failed - \
                        no records found in db"),
                }
            ).unwrap()))
        .unwrap();
    return Ok(response);
}
