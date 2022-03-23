//! # Handle all Client Requests
//!
//! Prometheus scrapes metrics at the endpoint:
//! ``https://API_ENDPOINT/metrics`` with a ``GET`` method.

use std::convert::Infallible;

use hyper::body;
use hyper::Body;
use hyper::Method;
use hyper::Response;

use crate::monitoring::metrics::record_monitoring_metrics_api_before;
use crate::monitoring::metrics::record_monitoring_metrics_api_after;
use crate::monitoring::metrics::handle_showing_metrics;
// metrics - end

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

    let mut processed_result: std::result::Result<Response<Body>, Infallible> = Ok(
        Response::new(
            Body::from(format!("prep"))));
    let (parts, body) = data.request.into_parts();
    let request_uri = parts.uri.path().clone();
    let request_method = parts.method.clone();
    match (
            request_method.clone(),
            request_uri) {
        (Method::POST, "/") => {
            if false {
                println!("{:?}", processed_result);
            }
            record_monitoring_metrics_api_before(
                &request_uri,
                "unknown",
                "post");
            let bytes = body::to_bytes(body).await.unwrap();
            let response_str = format!("\
                valid POST uri=/ data size={} bytes",
                bytes.len());
            processed_result = Ok(Response::new(Body::from(response_str)));
            return record_monitoring_metrics_api_after(
                &request_uri,
                "unknown",
                "post",
                processed_result);
        },
        (Method::POST, "/user") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "user",
                "post");
            let bytes = body::to_bytes(body).await.unwrap();
            processed_result = create_user(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &bytes).await;
            // this will check if the monitoring feature
            // was enabled or it returns the original
            // processed_result
            return record_monitoring_metrics_api_after(
                &request_uri,
                "user",
                "post",
                processed_result);
        },
        // end user creation
        (Method::DELETE, "/user") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "user",
                "delete");
            let bytes = body::to_bytes(body).await.unwrap();
            processed_result = delete_user(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
            return record_monitoring_metrics_api_after(
                &request_uri,
                "user",
                "delete",
                processed_result);
        },
        // end user deletion
        (Method::PUT, "/user") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "user",
                "put");
            let bytes = body::to_bytes(body).await.unwrap();
            processed_result = update_user(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
            return record_monitoring_metrics_api_after(
                &request_uri,
                "user",
                "put",
                processed_result);
        },
        // end user deletion
        (Method::POST, "/user/search") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "user",
                "search");
            let bytes = body::to_bytes(body).await.unwrap();
            processed_result = search_users(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
            return record_monitoring_metrics_api_after(
                &request_uri,
                "user",
                "search",
                processed_result);
        },
        // end user search
        (Method::POST, "/user/data") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "data",
                "upload");
            // tested without breaking the request into_parts() using:
            // let body_bytes = body::to_bytes(request.into_body()).await.unwrap();
            // multipart uploaded file handler
            processed_result = upload_user_data(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                body).await;
            return record_monitoring_metrics_api_after(
                &request_uri,
                "data",
                "upload",
                processed_result);
        },
        // end user data - create
        (Method::PUT, "/user/data") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "data",
                "put");
            let bytes = body::to_bytes(body).await.unwrap();
            processed_result = update_user_data(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
            return record_monitoring_metrics_api_after(
                &request_uri,
                "data",
                "put",
                processed_result);
        },
        // end user deletion
        (Method::POST, "/user/data/search") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "data",
                "search");
            let bytes = body::to_bytes(body).await.unwrap();
            processed_result = search_user_data(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
            return record_monitoring_metrics_api_after(
                &request_uri,
                "data",
                "search",
                processed_result);
        },
        // end user data - search via json containing optional dictionary parameters
        (Method::POST, "/user/password/reset") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "user",
                "create_otp");
            let bytes = body::to_bytes(body).await.unwrap();
            processed_result = create_otp(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
            return record_monitoring_metrics_api_after(
                &request_uri,
                "user",
                "create_otp",
                processed_result);
        },
        // end user password create a one-time-password record
        (Method::POST, "/user/password/change") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "user",
                "consume_otp");
            let bytes = body::to_bytes(body).await.unwrap();
            processed_result = consume_user_otp(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &parts.headers,
                &bytes).await;
            return record_monitoring_metrics_api_after(
                &request_uri,
                "user",
                "consume_otp",
                processed_result);
        },
        // end user password reset consuming user's one-time-password token
        (Method::POST, "/login") => {
            record_monitoring_metrics_api_before(
                &request_uri,
                "auth",
                "login");
            let bytes = body::to_bytes(body).await.unwrap();
            processed_result = login_user(
                &tracking_label,
                &data.config,
                &data.db_pool,
                &bytes).await;
            return record_monitoring_metrics_api_after(
                &request_uri,
                "auth",
                "login",
                processed_result);
        },
        // end user login
        (Method::GET, "/metrics") => {
            return handle_showing_metrics();
        },
        // end metrics
        (Method::GET, "/favicon.ico") => {
            let body = Body::from(format!("no favicon.ico"));
            processed_result = Ok(Response::new(body));
            return processed_result;
        },
        // end of favicon.ico
        _ => {
            if
                    request_method == Method::GET
                    && request_uri.contains("/user/verify") {
                record_monitoring_metrics_api_before(
                    &request_uri,
                    "user",
                    "consume_verify");
                let request_query_params = parts.uri.query().unwrap_or("");
                let full_url = format!("\
                    https://{}{request_uri}?{request_query_params}",
                    get_server_address("api"));
                processed_result = verify_user(
                    &tracking_label,
                    &data.config,
                    &data.db_pool,
                    &full_url).await;
                return record_monitoring_metrics_api_after(
                    &request_uri,
                    "user",
                    "consume_verify",
                    processed_result);
            }
            // end user verification
            else if
                    request_method == Method::GET
                    && request_uri.contains("/user/") {
                record_monitoring_metrics_api_before(
                    &request_uri,
                    "user",
                    "get");
                processed_result = get_user(
                    &tracking_label,
                    &data.config,
                    &data.db_pool,
                    &parts.headers,
                    request_uri).await;
                return record_monitoring_metrics_api_after(
                    &request_uri,
                    "user",
                    "get",
                    processed_result);
            }
            // end user get
            else {
                record_monitoring_metrics_api_before(
                    &request_uri,
                    "unknown",
                    "get");
                let reason = format!("\
                    unsupported method and uri \
                    https://{}{request_uri} \
                    method={request_method}",
                    data.config.server_address);
                let err_msg = format!("\
                    {{\"status\":400,\"reason\":\"{}\"}}",
                        reason);
                error!("{}", err_msg);
                let body = Body::from(err_msg);
                processed_result = Ok(Response::new(body));
                return record_monitoring_metrics_api_after(
                    &request_uri,
                    "unknown",
                    "get",
                    processed_result);
            }
        }
    }
}
