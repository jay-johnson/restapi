//! Module for validating a user's JWT token
//!
use postgres_native_tls::MakeTlsConnector;

use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;

use hyper::header::HeaderValue;
use hyper::HeaderMap;

use crate::core::core_config::CoreConfig;
use crate::jwt::api as jwt_api;
use crate::requests::models::user::get_user_by_id;

/// validate_user_token
///
/// Confirm the client's jwt from the header token key
/// (controlled by env var `TOKEN_HEADER=Bearer` as the default)
/// is valid with the following additional restriction(s):
///
/// ## validate_user_token restriction enforcing user must be active
///
/// The db `users.state` field for the user must
/// be *active* (`0`) to login.
///
/// # Arguments
///
/// * `tracking_label` - `*&str` - caller logging label
/// * `config` - [`CoreConfig`](crate::core::core_config::CoreConfig) -
///   server statics
/// * `conn` - [`PooledConnection`](bb8::PooledConnection) - established
///   db connection from the encrypted client threadpool
/// * `headers` - [`HeaderMap`](hyper::HeaderMap) - HTTP headers
///   as a map with the jwt
/// * `user_id` - `i32` - user id token in the `headers` must
///   match the db token for this user id
///
/// # Returns
///
/// ## validate_user_token on Success Returns
///
/// Ok(token: `String`)
///
/// ## validate_user_token on Failure Returns
///
/// Err(err_msg: `String`)
///
pub async fn validate_user_token(
    tracking_label: &str,
    config: &CoreConfig,
    conn: &PooledConnection<'_, PostgresConnectionManager<MakeTlsConnector>>,
    headers: &HeaderMap<HeaderValue>,
    user_id: i32,
) -> Result<String, String> {
    let token_header_key =
        std::env::var("TOKEN_HEADER").unwrap_or_else(|_| "Bearer".to_string());
    let (valid_user, user_model) =
        match get_user_by_id(tracking_label, user_id, conn).await {
            Ok(user_model) => {
                match user_model.state {
                    // only active users are allowed
                    // users.state = 0 (active)
                    0 => (true, user_model),
                    // users.state != 0 (inactive/invalid)
                    _ => {
                        let err_msg = format!(
                            "{tracking_label} user_id={user_id} \
                            is not active"
                        );
                        error!("{err_msg}");
                        return Err("INVALID".to_string());
                    }
                }
            }
            Err(err_msg) => {
                return Err(err_msg);
            }
        };
    if !valid_user {
        let err_msg = format!(
            "{tracking_label} token validation failed - user_id={user_id} \
            is not valid"
        );
        error!("{err_msg}");
        return Err("INVALID".to_string());
    }
    if headers.contains_key(&token_header_key) {
        let user_email = user_model.email.clone();
        let token = headers.get(&token_header_key).unwrap().to_str().unwrap();
        /*
        info!("{tracking_label} validating user {user_id} \
            token={token}");
        */
        match jwt_api::validate_token(
            tracking_label,
            token,
            &user_email,
            &config.decoding_key_bytes,
        )
        .await
        {
            Ok(_) => Ok(token.to_string()),
            Err(e) => {
                let err_msg = format!(
                    "{tracking_label} token validation failed for {user_email} \
                    err={e}"
                );
                error!("{err_msg}");
                Err("INVALID".to_string())
            }
        }
    } else {
        let err_msg = format!(
            "{tracking_label} \
            token validation failed missing header key={token_header_key} \
            for {user_id} request"
        );
        error!("{err_msg}");
        Err("INVALID".to_string())
    }
}
