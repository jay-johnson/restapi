//! ## Verify a User's email
//!
//! Consume a one-time-use verification token and change the user's ``users.verified`` value verified (``1``)
//!
//! - URL path: ``/user/verify``
//! - Method: ``GET``
//! - Handler: [`verify_user`](crate::requests::user::verify_user::verify_user)
//! - Request: [`ApiReqUserVerify`](crate::requests::user::verify_user::ApiReqUserVerify)
//! - Response: [`ApiResUserVerify`](crate::requests::user::verify_user::ApiResUserVerify)
//!

use std::convert::Infallible;

use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use hyper::Body;
use hyper::Response;

use serde::Serialize;
use serde::Deserialize;

use crate::core::core_config::CoreConfig;

use crate::utils::get_query_params_from_url::get_query_params_from_url;

use crate::requests::models::user::get_user_by_id;
use crate::requests::models::user_verify::get_user_verify_by_user_id;

use crate::requests::user::is_verification_enabled::is_verification_enabled;

/// ApiReqUserVerify
///
/// # Request Type For verify_user
///
/// Handles verifying a user's email with
/// a one-time-use verification token stored in the
/// `users_verified` record in the db.
///
/// # Overview
///
/// User email verification one-time-use token can expire
///
/// This type is the deserialized input for:
/// [`verify_user`](crate::requests::user::verify_user::verify_user]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `full_url` (`&str`) argument
/// on the
/// [`verify_user`](crate::requests::user::verify_user::verify_user)
/// function.
///
/// # Arguments
///
/// * `u` - `i32` - user id
/// * `t` - `String` -  the
///   `users_verified.password` field
/// * `e` - `Option<String>` - the
///   `users.email` field
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserVerify {
    // users.id
    pub u: i32,
    // users_verified.token for the user_id fk to users.id
    pub t: String,
    pub e: Option<String>,
}

/// ApiResUserVerify
///
/// # Response type for verify_user
///
/// Return user's db record
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`verify_user`](crate::requests::user::verify_user::verify_user]
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
/// * `state` - `i32` - user state
///   (`0` - active, `1` - inactive)
/// * `verified` - `i32` - user email verified
///   (`1` - verified)
/// * `role` - `String` - user role
/// * `msg` - `String` - help message
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResUserVerify {
    pub user_id: i32,
    pub email: String,
    pub state: i32,
    pub verified: i32,
    pub role: String,
    pub msg: String,
}

/// verify_user
///
/// Handles verifying a user's email (`users.email`)
/// record (in the `users_verified` table)
/// based off values in the query parameters on the hyper
/// [`Request`](hyper::Request)'s url
///
/// ## Overview Notes
///
/// This function only updates 1 `users_verified` record at a time.
///
/// # Arguments
///
/// * `tracking_label` - `&str` - caller logging label
/// * `_config` - [`CoreConfig`](crate::core::core_config::CoreConfig)
///   server statics and not used in request in this version
/// * `db_pool` - [`Pool`](bb8::Pool) - postgres client
///   db threadpool with required tls encryption
/// * `headers` - [`HeaderMap`](hyper::HeaderMap) -
///   hashmap containing headers in key-value pairs
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
/// * `full_url` - `&str` - the url in the
///   [`Request`](hyper::Request)
///
/// # Returns
///
/// ## verify_user on Success Returns
///
/// The updated `users` record from the db:
/// ([`ApiResUserVerify`](crate::requests::user::verify_user::ApiResUserVerify))
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserVerify`](crate::requests::user::verify_user::ApiResUserVerify)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `200` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## verify_user on Failure Returns
///
/// Note: user email verification can expire over time.
///       Any user can attempt to re-verify at any time.
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserVerify`](crate::requests::user::verify_user::ApiResUserVerify)
/// dictionary with a
/// `non-200` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn verify_user(
    tracking_label: &str,
    _config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    full_url: &str)
-> std::result::Result<Response<Body>, Infallible>
{
    // get query params as a hashmap
    let params_map = match get_query_params_from_url(
            &tracking_label,
            &full_url).await {
        Ok(params_map) => params_map,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserVerify {
                            user_id: -1,
                            email: format!(""),
                            state: -1,
                            verified: -1,
                            role: format!(""),
                            msg: format!("Missing required query params"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        },
    };

    /*
    info!("\
        {tracking_label} - \
        verify user - start - \
        params map: {:?}",
            params_map);
    */

    // get user_id from u=user_id
    let user_id: i32 = match params_map.get("u") {
        Some(user_id_str) => {
            let user_id: i32 = user_id_str.parse::<i32>().unwrap_or(-1);
            user_id
        },
        None => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserVerify {
                            user_id: -1,
                            email: format!(""),
                            state: -1,
                            verified: -1,
                            role: format!(""),
                            msg: format!("Missing required query param: user id"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        },
    };

    // get user_id from t=verify_token
    let verify_token: String = match params_map.get("t") {
        Some(verify_token) => {
            format!("{verify_token}")
        },
        None => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserVerify {
                            user_id: -1,
                            email: format!(""),
                            state: -1,
                            verified: -1,
                            role: format!(""),
                            msg: format!("\
                                User verify failed - please ensure \
                                the verify token is correct and reach out \
                                to support for additional help"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        },
    };

    if user_id <= 0 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserVerify {
                        user_id: -1,
                        email: format!(""),
                        state: -1,
                        verified: -1,
                        role: format!(""),
                        msg: format!("\
                            User verify failed - please ensure \
                            the user id must be a non-negative number \
                            and reach out to support for additional help"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    let verify_token_len = verify_token.len();
    if verify_token_len < 20 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserVerify {
                        user_id: -1,
                        email: format!(""),
                        state: -1,
                        verified: -1,
                        role: format!(""),
                        msg: format!("\
                            User verify failed - please ensure \
                            the verify token is valid \
                            ({verify_token_len} is too short) \
                            and reach out to support for additional help"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
    else if verify_token_len > 256 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserVerify {
                        user_id: -1,
                        email: format!(""),
                        state: -1,
                        verified: -1,
                        role: format!(""),
                        msg: format!("\
                            User verify failed - please ensure \
                            the verify token is valid \
                            ({verify_token_len} is too long) \
                            and reach out to support for additional help"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    let conn = db_pool.get().await.unwrap();

    // get the user
    let user_model = match get_user_by_id(
            &tracking_label,
            user_id,
            &conn).await {
        Ok(user_model) => user_model,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserVerify {
                            user_id: -1,
                            email: format!(""),
                            state: -1,
                            verified: -1,
                            role: format!(""),
                            msg: format!("\
                                User verify failed - please ensure \
                                the parameters are correct and reach out \
                                to support for additional help"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        },
    };

    // check that verification is enabled
    if ! is_verification_enabled() {
        let response = Response::builder()
            .status(200)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserVerify {
                        user_id: user_model.id,
                        email: user_model.email,
                        state: user_model.state,
                        verified: user_model.verified,
                        role: user_model.role,
                        msg: format!("User verification success"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    let user_email = user_model.email.clone();

    // is user in a non-active state
    if user_model.state != 0 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserVerify {
                        user_id: -1,
                        email: user_model.email,
                        state: user_model.state,
                        verified: user_model.verified,
                        role: user_model.role,
                        msg: format!("\
                            User {user_id} is inactive - \
                            not able to verify {user_email}"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    // already verified
    // prevent db hits when the user's already verified
    if user_model.verified != 0 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserVerify {
                        user_id: user_model.id,
                        email: user_model.email,
                        state: user_model.state,
                        verified: user_model.verified,
                        role: user_model.role,
                        msg: format!("\
                            User already verified"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    // get the verification record
    let user_verify_model = match get_user_verify_by_user_id(
            &tracking_label,
            user_id,
            &conn).await {
        Ok(uvm) => uvm,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserVerify {
                            user_id: -1,
                            email: format!(""),
                            state: -1,
                            verified: -1,
                            role: format!(""),
                            msg: format!("\
                                User verification record does not exist"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };

    let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let exp_vs_now_diff = now.signed_duration_since(user_verify_model.exp_date_utc);
    let exp_date_vs_now = exp_vs_now_diff.num_seconds();

    info!("\
        {tracking_label} - user {user_id} verifying exp_date={} \
        now={} \
        num_seconds_expired={exp_date_vs_now}s",
        user_verify_model.exp_date_utc.format("%Y-%m-%dT%H:%M:%SZ"),
        now.format("%Y-%m-%dT%H:%M:%SZ"));

    // check if the token is expired
    // now - exp_date > 0 == expired
    if exp_date_vs_now > 0 {
        let err_msg = format!("\
            {tracking_label} - user {user_id} \
            verify token {verify_token} \
            expired on: \
            exp_date={} \
            duration_since={exp_date_vs_now}s",
            user_verify_model.exp_date_utc);
        error!("{err_msg}");
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserVerify {
                        user_id: -1,
                        email: format!(""),
                        state: -1,
                        verified: -1,
                        role: format!(""),
                        msg: format!("\
                            user {user_email} verification has expired"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    let query = format!("\
        UPDATE \
            users_verified \
        SET \
            email = '{user_email}', \
            state = 1, \
            verify_date = '{now}' \
        WHERE \
            users_verified.user_id = {user_id} \
        RETURNING \
            users_verified.user_id,
            users_verified.token,
            users_verified.email,
            users_verified.state;");
    let stmt = conn.prepare(&query).await.unwrap();
    let query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => {
            info!("\
                {tracking_label} - \
                user {user_id} email {user_email} token verified");
            query_result
        },
        Err(e) => {
            let err_msg = format!("{e}");
            if
                    err_msg.contains("\
                        db error: ERROR: duplicate key value \
                        violates unique constraint")
                    && err_msg.contains("users_verified_email_key")
                    && err_msg.contains("already exists") {
                let response = Response::builder()
                    .status(400)
                    .body(Body::from(
                        serde_json::to_string(
                            &ApiResUserVerify {
                                user_id: -1,
                                email: format!(""),
                                state: -1,
                                verified: -1,
                                role: format!(""),
                                msg: format!("\
                                    User email is already \
                                    in use: {user_email}")
                            }
                        ).unwrap()))
                    .unwrap();
                return Ok(response);
            }
            else {
                let response = Response::builder()
                    .status(400)
                    .body(Body::from(
                        serde_json::to_string(
                            &ApiResUserVerify {
                                user_id: -1,
                                email: format!(""),
                                state: -1,
                                verified: -1,
                                role: format!(""),
                                msg: format!("\
                                    User update failed for user_id={user_id} {user_email} \
                                    with err='{err_msg}'")
                            }
                        ).unwrap()))
                    .unwrap();
                return Ok(response);
            }
        }
    };

    let query = format!("\
        UPDATE \
            users \
        SET \
            verified = 1 \
        WHERE \
            users.id = {user_id};");
    let stmt = conn.prepare(&query).await.unwrap();
    match conn.query(&stmt, &[]).await {
        Ok(_) => {
            info!("\
                {tracking_label} - \
                user {user_id} email {user_email} account verified");
        },
        Err(e) => {
            let err_msg = format!("{e}");
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserVerify {
                            user_id: -1,
                            email: format!(""),
                            state: -1,
                            verified: -1,
                            role: format!(""),
                            msg: format!("\
                                User table update failed for user verification \
                                user_id={user_id} {user_email}={verify_token} \
                                with err='{err_msg}'")
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };

    // must match up with RETURNING
    for row in query_result.iter() {
        let found_user_id: i32 = row.try_get("user_id").unwrap();
        let email: String = row.try_get("email").unwrap();
        let user_verify_state: i32 = row.try_get("state").unwrap();
        let response = Response::builder()
            .status(200)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserVerify {
                        user_id: found_user_id,
                        email: email.clone(),
                        state: user_model.state,
                        verified: user_verify_state,
                        role: user_model.role,
                        msg: format!("\
                            user {found_user_id} verified {email}"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    let response = Response::builder()
        .status(400)
        .body(Body::from(
            serde_json::to_string(
                &ApiResUserVerify {
                    user_id: -1,
                    email: format!(""),
                    state: -1,
                    verified: -1,
                    role: format!(""),
                    msg: format!("\
                        User update failed - user does \
                        not exist with user_id={user_id} email={user_email}")
                }
            ).unwrap()))
        .unwrap();
    return Ok(response);
}
