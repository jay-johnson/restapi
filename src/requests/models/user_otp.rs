use postgres_native_tls::MakeTlsConnector;

use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;

use serde::Deserialize;
use serde::Serialize;

/// ModelUserOtp
///
/// Representation in the db for a
/// user's one-time-use password reset token
///
/// Each user has 1 and only 1 `users_otp` record
///
/// # DB table
///
/// `users_otp`
///
/// # Arguments
///
/// * `id` - `i32` - `users_otp.id` in the db
/// * `user_id` - `i32` - `users.id` in the db
/// * `token` - `String` - one-time-use password token
/// * `exp_date_utc` - [`chrono::DateTime`](chrono::DateTime) -
///   the one-time-use password's expiration date in `Utc`
/// * `consumed_date_utc` -
///   [`chrono::DateTime`](chrono::DateTime)
///   most recent consume datetime in `Utc`
/// * `msg` - `String` - message for
///   helping debug from the client
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelUserOtp {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub email: String,
    pub state: i32,
    pub exp_date_utc: chrono::DateTime<chrono::Utc>,
    pub consumed_date_utc: Option<chrono::DateTime<chrono::Utc>>,
}

/// get_user_otp
///
/// Get the user's one-time-use password record
/// from the db
///
/// # Arguments
///
/// * `tracking_label` - `&str` - caller logging label
/// * `user_id` - `i32` - user id in the db
/// * `email` - `&str` - user's email address
/// * `token` - `&str` - user's one-time-use password token
/// * `conn` - [`PooledConnection`](bb8::PooledConnection) -
///   an established db connection from the
///   postgres client db threadpool
///
/// # Returns
///
/// ## get_user_otp on Success Returns
///
/// [`ModelUserOtp`](crate::requests::models::user_otp)
///
/// # Errors
///
/// Various `Err(String)` can be returned depending
/// on what breaks
///
pub async fn get_user_otp(
    tracking_label: &str,
    user_id: i32,
    email: &str,
    token: &str,
    conn: &PooledConnection<'_, PostgresConnectionManager<MakeTlsConnector>>,
) -> Result<ModelUserOtp, String> {
    // find all user by email and an active state where state == 0
    let query = format!(
        "SELECT \
            users_otp.id, \
            users_otp.user_id, \
            users_otp.token, \
            users_otp.email, \
            users_otp.state, \
            users_otp.exp_date, \
            users_otp.consumed_date, \
            users_otp.created_at \
        FROM \
            users_otp \
        WHERE \
            users_otp.user_id = {user_id} \
            AND \
            users_otp.token = '{token}' \
            AND \
            users_otp.email = '{email}' \
        LIMIT 1;"
    );
    // println!("{}", query);
    let stmt = conn.prepare(&query).await.unwrap();
    match conn.query(&stmt, &[]).await {
        Ok(query_result) => {
            if let Some(row) = query_result.first() {
                let found_db_id: i32 = row.try_get("id").unwrap();
                let found_user_id: i32 = row.try_get("user_id").unwrap();
                let found_token: String = row.try_get("token").unwrap();
                let found_email: String = row.try_get("email").unwrap();
                let found_state: i32 = row.try_get("state").unwrap();
                let found_exp_date_utc: chrono::DateTime<chrono::Utc> =
                    row.try_get("exp_date").unwrap();
                let found_consumed_date_utc: Option<
                    chrono::DateTime<chrono::Utc>,
                > = row.try_get("consumed_date").unwrap();
                return Ok(ModelUserOtp {
                    id: found_db_id,
                    user_id: found_user_id,
                    token: found_token,
                    email: found_email,
                    state: found_state,
                    exp_date_utc: found_exp_date_utc,
                    consumed_date_utc: found_consumed_date_utc,
                });
            }
            Err(format!(
                "{tracking_label} - \
                failed to find any user one-time-password \
                by user_id={user_id} \
                email={email}"
            ))
        }
        Err(e) => Err(format!(
            "{tracking_label} - \
                failed to find user one-time-password \
                by user_id={user_id} \
                with err='{e}'"
        )),
    }
}
