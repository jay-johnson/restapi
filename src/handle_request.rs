//! # Handle all Client Requests
//!

use std::convert::Infallible;

use hyper::body;
use hyper::Body;
use hyper::Method;
use hyper::Response;

use crate::core::server::core_task_item::CoreTaskItem;

// request utils
use crate::utils::get_server_address::get_server_address;

// request handlers

// auth requests
use crate::requests::auth::login_user::login_user;

// user requests
use crate::requests::user::consume_user_otp::consume_user_otp;
use crate::requests::user::create_user::create_user;
use crate::requests::user::create_otp::create_otp;
use crate::requests::user::delete_user::delete_user;
use crate::requests::user::get_user::get_user;
use crate::requests::user::search_users::search_users;
use crate::requests::user::search_user_data::search_user_data;
use crate::requests::user::update_user::update_user;
use crate::requests::user::update_user_data::update_user_data;
use crate::requests::user::upload_user_data::upload_user_data;
use crate::requests::user::verify_user::verify_user;

/// handle_request
///
/// The url routing handler for all api requests.
///
/// # Arguments
///
/// * `data` - [`CoreTaskItem`](crate::core::server::core_task_item::CoreTaskItem)
///
pub async fn handle_request(
    data: CoreTaskItem)
-> std::result::Result<Response<Body>, Infallible>
{
    /*
    let tracking_label = format!("\
        {} - {:?} - server handler",
            data.config.label,
            std::thread::current().id());
    */
    let tracking_label = format!("\
        {}",
            data.config.label);

    // Handle requests here

    /*
    //
    // debug client ciphers with this section
    //
    let remote_addr = data.remote_addr.clone();
    let cipher_suite = data.tls_info.clone().as_ref().unwrap().ciphersuite;

    info!("\
        {tracking_label} - handle request - \
        uri={:?} \
        method={:?} \
        remote_addr={remote_addr} \
        ciphers={:?}",
        data.request.uri(),
        data.request.method(),
        remote_addr,
        cipher_suite);
    */

    let (parts, body) = data.request.into_parts();
    let request_uri = parts.uri.path().clone();
    let request_method = parts.method.clone();
    match (
            request_method.clone(),
            request_uri) {
        (Method::POST, "/") => {
            let bytes = body::to_bytes(body).await.unwrap();
            let response_str = format!("\
                valid POST uri=/ data size={} bytes",
                bytes.len());
            return Ok(Response::new(Body::from(response_str)));
        },
        (Method::POST, "/user") => {
            let bytes = body::to_bytes(body).await.unwrap();
            return create_user(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &bytes).await;
        },
        // end user creation
        (Method::DELETE, "/user") => {
            let bytes = body::to_bytes(body).await.unwrap();
            return delete_user(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
        },
        // end user deletion
        (Method::PUT, "/user") => {
            let bytes = body::to_bytes(body).await.unwrap();
            return update_user(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
        },
        // end user deletion
        (Method::POST, "/user/search") => {
            let bytes = body::to_bytes(body).await.unwrap();
            return search_users(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
        },
        // end user search
        (Method::POST, "/user/data") => {
            // tested without breaking the request into_parts() using:
            // let body_bytes = body::to_bytes(request.into_body()).await.unwrap();
            // multipart uploaded file handler
            return upload_user_data(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                body).await;
        },
        // end user data - create
        (Method::PUT, "/user/data") => {
            let bytes = body::to_bytes(body).await.unwrap();
            return update_user_data(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
        },
        // end user deletion
        (Method::POST, "/user/data/search") => {
            let bytes = body::to_bytes(body).await.unwrap();
            return search_user_data(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
        },
        // end user data - search via json containing optional dictionary parameters
        (Method::POST, "/user/password/reset") => {
            let bytes = body::to_bytes(body).await.unwrap();
            return create_otp(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
        },
        // end user password create a one-time-password record
        (Method::POST, "/user/password/change") => {
            let bytes = body::to_bytes(body).await.unwrap();
            return consume_user_otp(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
        },
        // end user password reset consuming user's one-time-password token
        (Method::POST, "/login") => {
            let bytes = body::to_bytes(body).await.unwrap();
            return login_user(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &bytes).await;
        },
        // end user login
        _ => {
            if
                    request_method == Method::GET
                    && request_uri.contains("/user/verify") {
                let request_query_params = parts.uri.query().unwrap_or("");
                let full_url = format!("\
                    https://{}{request_uri}?{request_query_params}",
                    get_server_address("api"));
                return verify_user(
                    &tracking_label,
                    &data.config,
                    &data.db_pool,
                    &full_url).await;
            }
            // end user verification
            else if
                    request_method == Method::GET
                    && request_uri.contains("/user/") {
                return get_user(
                    &tracking_label,
                    &data.config,
                    &data.db_pool,
                    &parts.headers,
                    request_uri).await;
            }
            // end user get
            else {
                let err_msg = format!("\
                    {tracking_label} - \
                    handle request failure - \
                    unsupported method and uri - \
                    https://{}{request_uri} \
                    method={request_method}",
                    data.config.server_address);
                error!("{}", err_msg);
                let body = Body::from(err_msg);
                return Ok(Response::new(body));
            }
        }
    }
}
