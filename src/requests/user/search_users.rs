//! ## Search Users in the db
//!
//! Search for matching ``users`` records in the db
//!
//! - URL path: ``/user/search``
//! - Method: ``POST``
//! - Handler: [`search_users`](crate::requests::user::search_users::search_users)
//! - Request: [`ApiReqUserSearch`](crate::requests::user::search_users::ApiReqUserSearch)
//! - Response: [`ApiResUserSearch`](crate::requests::user::search_users::ApiResUserSearch)
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

use crate::requests::user::get_user::ApiResUserGet;
use crate::requests::auth::validate_user_token::validate_user_token;

/// ApiReqUserSearch
///
/// # Request Type For search_users
///
/// Handles searching for many `users`
/// record(s) from the db with optional filters
///
/// This type is the deserialized input for:
/// [`search_users`](crate::requests::user::search_users::search_users]
///
/// # Usage
///
/// This type is constructed from the deserialized
/// `bytes` (`&[u8]`) argument
/// on the
/// [`search_users`](crate::requests::user::search_users::search_users)
/// function.
///
/// # Arguments
///
/// * `user_id` - `i32` - user id
/// * `email` - `String` - filter by
///   `users.email` with `ILIKE`
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiReqUserSearch {
    pub user_id: i32,
    pub email: String,
}

/// ApiResUserSearch
///
/// # Response type for search_users
///
/// Contains matching `users` records from the
/// db based off the POST-ed filters in the type:
/// [`ApiReqUserSearch`](crate::requests::user::search_users::ApiReqUserSearch)
///
/// # Usage
///
/// This type is the serialized output for the function:
/// [`search_users`](crate::requests::user::search_users::search_users]
/// and contained within the
/// hyper [`Body`](hyper::Body)
/// of the
/// hyper [`Response`](hyper::Response)
/// sent back to the client.
///
/// # Arguments
///
/// * `users` - Vec<[`ApiResUserGet`](crate::requests::user::get_user::ApiResUserGet)> -
///   list of matching `users` record(s)
/// * `msg` - `String` - help message
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ApiResUserSearch {
    pub users: Vec<ApiResUserGet>,
    pub msg: String,
}

/// search_users
///
/// Search for matching `users_data` records by the POST-ed
/// [`ApiReqUserSearch`](crate::requests::user::search_users::ApiReqUserSearch)
/// (filters) and return a list of
/// [`ApiResUserGet`](crate::requests::user::get_user::ApiResUserGet)
/// within the
/// [`ApiResUserSearch`](crate::requests::user::search_users::ApiResUserSearch)
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
/// * `headers` - [`HeaderMap`](hyper::HeaderMap) -
///   hashmap containing headers in key-value pairs
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
/// * `bytes` - `&[u8]` - received bytes from the hyper
///   [`Request`](hyper::Request)'s [`Body`](hyper::Body)
///
/// # Returns
///
/// ## search_users on Success Returns
///
/// List of matching `users` records from the db
///
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiReqUserSearch`](crate::requests::user::search_users)
/// dictionary within the
/// [`Body`](hyper::Body) and a
/// `200` HTTP status code
///
/// Ok([`Response`](hyper::Response))
///
/// # Errors
///
/// ## search_users on Failure Returns
///
/// All errors return as a
/// hyper [`Response`](hyper::Response)
/// containing a json-serialized
/// [`ApiReqUserSearch`](crate::requests::user::search_users)
/// dictionary with a
/// `non-200` HTTP status code
///
/// Err([`Response`](hyper::Response))
///
pub async fn search_users(
    tracking_label: &str,
    config: &CoreConfig,
    db_pool: &Pool<PostgresConnectionManager<MakeTlsConnector>>,
    headers: &HeaderMap<HeaderValue>,
    bytes: &[u8])
-> std::result::Result<Response<Body>, Infallible>
{
    let user_object: ApiReqUserSearch = match serde_json::from_slice(&bytes) {
        Ok(uo) => uo,
        Err(_) => {
            let response = Response::builder()
                .status(400)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserSearch {
                            users: Vec::new(),
                            msg: format!("Missing user_id and email to search"),
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };
    let user_id: i32 = user_object.user_id.clone();
    let user_email: String = user_object.email.clone();

    if
            user_id < 1
            || user_email == "" {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserSearch {
                        users: Vec::new(),
                        msg: format!("Missing user_id and email to search"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    if user_email.len() < 3 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserSearch {
                        users: Vec::new(),
                        msg: format!("User search requires at least 3 characters"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }

    info!("{tracking_label} - searching user_id={user_id} email={user_email}");

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
                    &ApiResUserSearch {
                        users: Vec::new(),
                        msg: format!("User search failed due to invalid token"),
                    }
                ).unwrap()))
            .unwrap();
            return Ok(response);
        }
    };

    // find all user by email and an active state where state == 0
    let get_query = format!("\
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
            users.email \
        ILIKE \
            '%{}%' \
        ORDER BY \
            users.created_at \
        DESC \
        LIMIT 100",
        user_email);
    let stmt = conn.prepare(&get_query).await.unwrap();
    let mut query_result: Vec<data_row> = Vec::with_capacity(5);
    if false {
        println!("{}", query_result.len());
    }
    query_result = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let err_msg = format!("{}", e);
            let response = Response::builder()
                .status(500)
                .body(Body::from(
                    serde_json::to_string(
                        &ApiResUserSearch {
                            users: Vec::new(),
                            msg: format!("User search failed for user_id={user_id} email={user_email} with err='{err_msg}'")
                        }
                    ).unwrap()))
                .unwrap();
            return Ok(response);
        }
    };
    let mut row_list: Vec<ApiResUserGet> = Vec::with_capacity(100);
    for row in query_result.iter() {
        let id: i32 = row.try_get("id").unwrap();
        let email: String = row.try_get("email").unwrap();
        let user_state: i32 = row.try_get("state").unwrap();
        let user_verified: i32 = row.try_get("verified").unwrap();
        let role: String = row.try_get("role").unwrap();
        row_list.push(ApiResUserGet {
            user_id: id,
            email: email,
            state: user_state,
            verified: user_verified,
            role: role,
            msg: String::from(""),
        });
    }
    if row_list.len() == 0 {
        let response = Response::builder()
            .status(400)
            .body(Body::from(
                serde_json::to_string(
                    &ApiResUserSearch {
                        users: Vec::new(),
                        msg: format!("no users found"),
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
                    &ApiResUserSearch {
                        users: row_list,
                        msg: format!("success"),
                    }
                ).unwrap()))
            .unwrap();
        return Ok(response);
    }
}
