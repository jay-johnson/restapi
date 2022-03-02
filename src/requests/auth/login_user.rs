//! ## User Login
//!
//! Log the user in and get a json web token (jwt) back for authentication on subsequent client requests
//!
//! - URL path: ``/login``
//! - Method: ``POST``
//! - Handler: [`login`](crate::requests::auth::login_user::login_user)
//! - Request: [`ApiReqUserLogin`](crate::requests::auth::login_user::ApiReqUserLogin)
//! - Response: [`ApiResUserLogin`](crate::requests::auth::login_user::ApiResUserLogin)
//!

use std::convert::Infallible;

use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use hyper::Body;
use hyper::Response;

use serde::Serialize;
use serde::Deserialize;

use argon2::Config as argon_config;
use argon2::hash_encoded as argon_hash_encoded;

use crate::core::core_config::CoreConfig;

use crate::requests::auth::create_user_token::create_user_token;

use crate::requests::user::is_verification_required::is_verification_required;

/// ApiReqUserLogin
///
/// # Request Type For login_user
///
/// User login request
///
/// This type is the deserialized input for:
/// [`login_user`](crate::requests::auth::login_user::login_user]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// [`login_user`](crate::requests::auth::login_user::login_user)
/// function.
///
/// # Arguments
///
/// * `email` - `String` - unique user email
/// * `password` - `String` - user password
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserLogin {
    pub email: String,
    pub password: String,
}

/// ApiResUserLogin
///
/// # Response type for login_user
///
/// Return user's db record with new encrypted jwt
/// from the `users_tokens` db table (on success).
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`login_user`](crate::requests::auth::login_user::login_user]
/// and contained within the
/// hyper [`Body`](hyper::Body)
/// of the
/// hyper [`Response`](hyper::Response)
/// sent back to the client.
///
/// # Arguments
///
/// * `user_id` - `i32` - existing user id
/// * `email` - `String` - unique user email
/// * `state` - `i32` - user state code (`0` = an active user, `1` = not active)
/// * `verified` - `i32` - is user email verified (`0` = not verified, `1` = verified)
/// * `role` - `String` - user role
/// * `token` - `String` - encrypted jwt
/// * `msg` - `String` - error message
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResUserLogin {
    pub user_id: i32,
    pub email: String,
    pub state: i32,
    pub verified: i32,
    pub role: String,
    pub token: String,
    pub msg: String,
}

/// login_user
///
/// Handler for logging a user into the system.
///
/// Validates the user credentials with `argon2`
/// and creates a new, encrypted jwt for the user.
///
/// ## login_user restriction enforcing user must be active
///
/// The db `users.state` field for the user must
/// be *active* (`0`) to login.
///
/// # Arguments
///
/// * `tracking_label` - `&str` - logging label for caller
/// * `config` - [`CoreConfig`](crate::core::core_config::CoreConfig)
/// * `db_pool` - [`Pool`](bb8::Pool) - postgres client
///   db threadpool with required tls encryption
/// * `bytes` - `&[u8]` - bytes received from the hyper server
///
/// # Returns
///
/// ## login_user on Success Returns
///
/// HTTP status code `201` with `ApiResUserLogin` in the
/// hyper [`Response`](hyper::Response)
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## login_user on Failure Returns
///
/// `non-201` HTTP status code with `ApiResUserLogin` in the
/// hyper [`Response`](hyper::Response)
///
/// No `Err` called. All internal errors mapped
/// to HTML status codes with a `String` *msg* value
/// returned in the serialized
/// json response
///
/// Err([`Infallible`](std::convert::Infallible))
///
pub async fn login_user(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    bytes: &[u8])
-> std::result::Result<Response<Body>, Infallible>
{
    // deserialize into a type
    let user_object: ApiReqUserLogin = match serde_json::from_slice(&bytes) {
        Ok(uo) => uo,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserLogin {
                            user_id: -1,
                            email: String::from(""),
                            state: -1,
                            verified: -1,
                            role: String::from(""),
                            token: String::from(""),
                            msg: format!("\
                                Login failed - please ensure \
                                email and password \
                                were set correctly in the request"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };

    // salt the password
    let argon_config = argon_config::default();
    let hash = argon_hash_encoded(
        user_object.password.as_bytes(),
        &config.server_password_salt,
        &argon_config).unwrap();

    // find all user by email and an active state where state == 0
    let query = format!("\
        SELECT \
            users.id, \
            users.email, \
            users.password, \
            users.state, \
            users.verified, \
            users.role \
        FROM \
            users \
        WHERE \
            users.email = '{}' \
        AND \
            users.state = 0 \
        LIMIT 1;",
        &user_object.email);
    let conn = db_pool.get().await.unwrap();
    let stmt = conn.prepare(&query).await.unwrap();
    let query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let err_msg = format!("{e}");
            let response = Response::builder()
                .status(500)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserLogin {
                            user_id: -1,
                            email: String::from(""),
                            state: -1,
                            verified: -1,
                            role: String::from(""),
                            token: String::from(""),
                            msg: format!("\
                                User login failed for email={} with err='{err_msg}'",
                                user_object.email)
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };
    let mut row_list: Vec<(i32, String, String, i32, i32, String)> = Vec::with_capacity(1);
    for row in query_result.iter() {
        let id: i32 = row.try_get("id").unwrap();
        let email: String = row.try_get("email").unwrap();
        let password: String = row.try_get("password").unwrap();
        if password != hash {
            // error!("{tracking_label} - BAD LOGIN:\n{password}\n!=\n{hash}");
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserLogin {
                            user_id: -1,
                            email: String::from(""),
                            state: -1,
                            verified: -1,
                            role: String::from(""),
                            token: String::from(""),
                            msg: format!("\
                                User login failed - invalid password")
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
        let user_state: i32 = row.try_get("state").unwrap();
        let user_verified: i32 = row.try_get("verified").unwrap();

        // if user verification is enabled and the user
        // has not verified - reject the auth
        if is_verification_required() && user_verified != 1 {
            let err_msg = format!("\
                User login rejected - the email address: {email} \
                is not verified");
            error!("\
                {tracking_label} - {err_msg}");
            let response = Response::builder()
                .status(401)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserLogin {
                            user_id: -1,
                            email: String::from(""),
                            state: -1,
                            verified: -1,
                            role: String::from(""),
                            token: String::from(""),
                            msg: err_msg,
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }

        let role: String = row.try_get("role").unwrap();
        row_list.push((
            id,
            email,
            password,
            user_state,
            user_verified,
            role
        ))
    }
    if row_list.len() == 0 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserLogin {
                        user_id: -1,
                        email: String::from(""),
                        state: -1,
                        verified: -1,
                        role: String::from(""),
                        token: String::from(""),
                        msg: format!("\
                            User login failed - user does not exist with email={}",
                            user_object.email)
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
    else {
        let user_id = row_list[0].0.clone();
        let user_email = row_list[0].1.clone();
        let user_token = match create_user_token(
                tracking_label,
                config,
                &conn,
                &user_email,
                user_id).await {
            Ok(user_token) => user_token,
            Err(_) => {
                let response = Response::builder()
                    .status(400)
                    .body(Body::from(
                        serde_json::to_string(
                            &ApiResUserLogin {
                                user_id: -1,
                                email: String::from(""),
                                state: -1,
                                verified: -1,
                                role: String::from(""),
                                token: String::from(""),
                                msg: format!("\
                                    User login failed - unable to create user token for user_id={user_id} email={}",
                                    user_object.email)
                            }
                        ).unwrap()))
                    .unwrap();
                return Ok(response);
            }
        };
        let response = Response::builder()
            .status(201)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserLogin {
                        user_id: user_id,
                        email: user_email,
                        state: row_list[0].3.clone(),
                        verified: row_list[0].4.clone(),
                        role: row_list[0].5.clone(),
                        token: user_token,
                        msg: format!("success"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
}
