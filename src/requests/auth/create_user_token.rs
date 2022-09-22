//! Create a user's JWT token
//!
use postgres_native_tls::MakeTlsConnector;

use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;

use crate::jwt::api as jwt_api;

use crate::core::core_config::CoreConfig;

/// create_user_token
///
/// Create a signed jwt for the ``user_id`` and ``user_email``
/// and store it in postgres with an expiration date
///
/// # Arguments
///
/// * `tracking_label` - `&str` - logging label for caller
/// * `config` - [`CoreConfig`](crate::core::core_config::CoreConfig) -
///   server config
/// * `conn` - [`PooledConnection`](bb8::PooledConnection) -
///   established db connection from the threadpool
/// * `user_email` - `&str` - user's email
/// * `user_id` - `i32` - user's database id
///
/// # Returns
///
/// ## create_user_token on Success Returns
///
/// Ok(token: `String`)
///
/// ## create_user_token on Failure Returns
///
/// Err(err_msg: `String`)
///
pub async fn create_user_token(
    tracking_label: &str,
    config: &CoreConfig,
    conn: &PooledConnection<'_, PostgresConnectionManager<MakeTlsConnector>>,
    user_email: &str,
    user_id: i32,
) -> Result<String, String> {
    info!("{tracking_label} creating user {user_id} token");
    let new_token = match jwt_api::create_token(
        tracking_label,
        user_email,
        &config.encoding_key_bytes,
    )
    .await
    {
        Ok(token) => token,
        Err(err_msg) => {
            error!(
                "{tracking_label} failed to create user {user_id} {user_email} \
                token with jwt_api call - err_msg='{err_msg}'"
            );
            return Err("INVALID".to_string());
        }
    };
    let insert_query = format!(
        "INSERT INTO \
            users_tokens (\
                user_id, \
                token, \
                state) \
        VALUES (\
            {user_id}, \
            '{new_token}', \
            0)"
    );
    let stmt = conn.prepare(&insert_query).await.unwrap();
    let _ = match conn.query(&stmt, &[]).await {
        Ok(_query_result) => _query_result,
        Err(e) => {
            let err_msg = format!("{e}");
            error!(
                "{tracking_label} db failed to add new user token for user \
                {user_id} \
                email={user_email} \
                with err='{err_msg}'"
            );
            return Err("INVALID".to_string());
        }
    };
    Ok(new_token)
}
