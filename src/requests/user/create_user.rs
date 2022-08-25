//! ## Create User
//!
//! Create a single ``users`` record for the new user
//!
//! - URL path: ``/user``
//! - Method: ``POST``
//! - Handler: [`create_user`](crate::requests::user::create_user::create_user)
//! - Request: [`ApiReqUserCreate`](crate::requests::user::create_user::ApiReqUserCreate)
//! - Response: [`ApiResUserCreate`](crate::requests::user::create_user::ApiResUserCreate)
//!

use std::convert::Infallible;

use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use hyper::Body;
use hyper::Response;

use serde::Deserialize;
use serde::Serialize;

use argon2::hash_encoded as argon_hash_encoded;
use argon2::Config as argon_config;

use crate::core::core_config::CoreConfig;

use crate::utils::get_server_address::get_server_address;

use crate::requests::auth::create_user_token::create_user_token;
use crate::requests::auth::login_user::ApiResUserLogin;

use crate::requests::user::is_verification_enabled::is_verification_enabled;
use crate::requests::user::upsert_user_verification::upsert_user_verification;

/// ApiReqUserCreate
///
/// # Request Type For create_user
///
/// Create a new user in the db
///
/// This type is the deserialized input for:
/// [`create_user`](crate::requests::user::create_user::create_user]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// [`create_user`](crate::requests::user::create_user::create_user)
/// function.
///
/// # Arguments
///
/// * `email` - `String` - user email
/// * `password` - `String` - new user password
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserCreate {
    pub email: String,
    pub password: String,
}

/// ApiResUserCreate
///
/// # Response type for create_user
///
/// Return users's db record with encrypted jwt
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`create_user`](crate::requests::user::create_user::create_user]
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
/// * `state` - `i32` - user state where
///   (`0` - active, `1` - inactive)
/// * `role` - `String` - user role
/// * `token` - `String` - user jwt
/// * `msg` - `String` - help message
///
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ApiResUserCreate {
    pub user_id: i32,
    pub email: String,
    pub state: i32,
    pub role: String,
    pub token: String,
    pub msg: String,
}

/// create_user
///
/// Create a new user from the deserialized
/// [`ApiReqUserCreate`](crate::requests::user::create_user::ApiReqUserCreate)
/// json values from the `bytes` argument.
///
/// Also create a new user jwt and
/// email verification record (if enabled).
///
/// # Arguments
///
/// * `tracking_label` - `&str` - caller logging label
/// * `config` - [`CoreConfig`](crate::core::core_config::CoreConfig)
/// * `db_pool` - [`Pool`](bb8::Pool) - postgres client
///   db threadpool with required tls encryption
/// * `bytes` - `&[u8]` - received bytes from the hyper
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
///
/// # Returns
///
/// ## create_user on Success Returns
///
/// The new user record from the db and a jwt for auto-auth.
/// (token created by
/// [`create_user_token`](crate::requests::auth::create_user_token::create_user_token)
/// )
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserCreate`](crate::requests::user::create_user::ApiResUserCreate)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `201` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## create_user on Failure Returns
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiResUserCreate`](crate::requests::user::create_user::ApiResUserCreate)
/// dictionary with a
/// `non-201` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn create_user(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    bytes: &[u8],
) -> std::result::Result<Response<Body>, Infallible> {
    let user_object: ApiReqUserCreate = serde_json::from_slice(bytes).unwrap();

    if user_object.password.len() < 4 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(&ApiResUserCreate {
                    user_id: -1,
                    email: "".to_string(),
                    state: -1,
                    role: "".to_string(),
                    token: "".to_string(),
                    msg: ("User password must be more than 4 characters")
                        .to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        return Ok(response);
    }

    let mut user_role = "user";
    if user_object.email == "admin@email.com" {
        user_role = "admin";
    }

    let user_verification_enabled = is_verification_enabled();
    let user_start_state_value = 0;
    let user_verified_value = match user_verification_enabled {
        true => 0,
        false => 1,
    };

    // salt the user's password
    let argon_config = argon_config::default();
    let hash = argon_hash_encoded(
        user_object.password.as_bytes(),
        &config.server_password_salt,
        &argon_config,
    )
    .unwrap();

    let insert_query = format!(
        "INSERT INTO \
            users (\
                email, \
                password, \
                state, \
                verified, \
                role) \
        VALUES (\
            '{}', \
            '{hash}', \
            {user_start_state_value}, \
            {user_verified_value}, \
            '{user_role}') \
        RETURNING \
            users.id, \
            users.email, \
            users.password, \
            users.state, \
            users.verified, \
            users.role;",
        user_object.email
    );
    let conn = db_pool.get().await.unwrap();
    let stmt = conn.prepare(&insert_query).await.unwrap();
    let query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let err_msg = format!("{e}");
            if err_msg.contains("duplicate key value violates") {
                let response = Response::builder()
                    .status(400)
                    .body(Body::from(
                        serde_json::to_string(&ApiResUserCreate {
                            user_id: -1,
                            email: "".to_string(),
                            state: -1,
                            role: "".to_string(),
                            token: "".to_string(),
                            msg: format!(
                                "User email {} already registered",
                                user_object.email
                            ),
                        })
                        .unwrap(),
                    ))
                    .unwrap();
                return Ok(response);
            } else {
                let response = Response::builder()
                    .status(500)
                    .body(Body::from(
                        serde_json::to_string(
                            &ApiResUserCreate {
                                user_id: -1,
                                email: "".to_string(),
                                state: -1,
                                role: "".to_string(),
                                token: "".to_string(),
                                msg: format!(
                                    "User creation failed for email={} with err='{err_msg}'",
                                        user_object.email)
                        }).unwrap()))
                    .unwrap();
                return Ok(response);
            }
        }
    };

    let mut row_list: Vec<(i32, String, String, i32, i32, String)> =
        Vec::with_capacity(1);
    for row in query_result.iter() {
        let id: i32 = row.try_get("id").unwrap();
        let email: String = row.try_get("email").unwrap();
        let password: String = row.try_get("password").unwrap();
        if password != hash {
            error!("BAD PASSWORD FOUND DURING USER CREATION:\npassword=\n{password}\n!=\nsalt=\n{hash}");
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(&ApiResUserLogin {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        token: "".to_string(),
                        msg: ("User login failed - invalid password")
                            .to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
            return Ok(response);
        }
        let user_state: i32 = row.try_get("state").unwrap();
        let user_verified_db: i32 = row.try_get("verified").unwrap();
        let role: String = row.try_get("role").unwrap();
        row_list.push((id, email, password, user_state, user_verified_db, role))
    }
    if row_list.is_empty() {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserLogin {
                        user_id: -1,
                        email: "".to_string(),
                        state: -1,
                        verified: -1,
                        role: "".to_string(),
                        token: "".to_string(),
                        msg: format!(
                            "User creation failed - user does not exist with email={}",
                                user_object.email)
                    }
                ).unwrap()))
            .unwrap();
        Ok(response)
    } else {
        let user_id = row_list[0].0;
        let user_email = row_list[0].1.clone();
        let user_token = match create_user_token(
            tracking_label,
            config,
            &conn,
            &user_email,
            user_id,
        )
        .await
        {
            Ok(user_token) => user_token,
            Err(_) => {
                let response = Response::builder()
                    .status(500)
                    .body(Body::from(
                        serde_json::to_string(
                            &ApiResUserLogin {
                                user_id: -1,
                                email: "".to_string(),
                                state: -1,
                                verified: -1,
                                role: "".to_string(),
                                token: "".to_string(),
                                msg: format!("User token creation failed - {user_id} {user_email}"),
                            }
                        ).unwrap()))
                    .unwrap();
                return Ok(response);
            }
        };
        if user_verification_enabled {
            match upsert_user_verification(
                tracking_label,
                user_id,
                &user_email,
                true, // is new user flag
                0,    // not verified
                &conn,
            )
            .await
            {
                Ok(verification_token) => {
                    info!(
                        "{tracking_label} - verify token created user={user_id} \
                        {user_email} - verify url:\
                        curl -ks \
                        \"https://{}/user/verify?u={user_id}&t={verification_token}\" \
                        | jq",
                            get_server_address("api"));
                }
                Err(e) => {
                    error!(
                        "{tracking_label} - \
                        failed to generate verify token for user {user_id} \
                        {user_email} with err='{e}'"
                    );
                }
            };
        }
        let response = Response::builder()
            .status(201)
            .body(Body::from(
                serde_json::to_string(&ApiResUserLogin {
                    user_id,
                    email: user_email,
                    state: row_list[0].3,
                    verified: row_list[0].4,
                    role: row_list[0].5.clone(),
                    token: user_token,
                    msg: "success".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        Ok(response)
    }
}
