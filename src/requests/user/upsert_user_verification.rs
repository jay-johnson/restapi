//! Module for upsert-ing a user's email verification record
//! in the postgres db
//!
use postgres_native_tls::MakeTlsConnector;

use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;

use chrono::Duration;
use chrono::Utc;

use crate::requests::user::is_verification_enabled::is_verification_enabled;
use crate::utils::get_uuid::get_uuid;

/// upsert_user_verification
///
/// Upsert the user's `users_verified` record in the db
/// and insert/update the user email verification token
/// value. Also set the
/// `users_verified.exp_date`
/// to an expiration date in future (Utc)
/// based off the environment variable
/// `USER_EMAIL_VERIFICATION_EXP_IN_SECONDS`.
///
/// # Usage
///
/// ## Environment Variables
///
/// Email verification expiration date can be changed with
///
/// ```bash
/// # 1 second
/// export USER_EMAIL_VERIFICATION_EXP_IN_SECONDS=1
/// # 30 days
/// export USER_EMAIL_VERIFICATION_EXP_IN_SECONDS=2592000
/// ```
///
/// # Arguments
///
/// * `tracking_label` - `&str` - caller logging label
/// * `user_id` - `i32` - user id
/// * `email` - `&str` - email address
/// * `is_new_user` - `bool` - flag to allow skipping
///   updating the `users.email`
/// * `verified` - `i32` - change the email verification
///   with default unverified (`1`) and verified (`1`)
/// * `conn` - [`PooledConnection`](bb8::PooledConnection) -
///   an established db connection from the
///   postgres client db threadpool
///
/// # Returns
///
/// ## upsert_user_verification on Success Returns
///
/// Ok(`String`)
///
/// # Errors
///
/// ## upsert_user_verification on Failure Returns
///
/// Err(`String`)
///
pub async fn upsert_user_verification(
    tracking_label: &str,
    user_id: i32,
    email: &str,
    is_new_user: bool,
    verified: i32,
    conn: &PooledConnection<'_, PostgresConnectionManager<MakeTlsConnector>>,
) -> Result<String, String> {
    // create the new email verification token value
    let token = get_uuid();
    let user_verified_value = match is_verification_enabled() {
        true => 0,
        false => 1,
    };
    let user_verification_expiration_in_seconds_str =
        std::env::var("USER_EMAIL_VERIFICATION_EXP_IN_SECONDS")
            .unwrap_or_else(|_| "2592000".to_string());
    let user_verification_expiration_in_seconds: i64 =
        user_verification_expiration_in_seconds_str
            .parse::<i64>()
            .unwrap();
    let now = Utc::now();
    // https://docs.rs/chrono/0.4.19/chrono/struct.Duration.html#method.seconds
    let verification_expiration_timestamp =
        now + Duration::seconds(user_verification_expiration_in_seconds);

    // https://www.postgresqltutorial.com/postgresql-upsert/
    //    INSERT INTO customers (name, email)
    //    VALUES('Microsoft','hotline@microsoft.com')
    //    ON CONFLICT (name)
    //    DO
    //    UPDATE SET
    // vs
    //    if you want to concat emails the user has verified with upsert:
    //    UPDATE SET email = EXCLUDED.email || ';' || users_verified.email;

    // set the users.email + users.verified = 0
    if !is_new_user {
        let query = format!(
            "UPDATE \
                users \
            SET \
                email = '{email}', \
                verified = {verified} \
            WHERE \
                users.id = {user_id};"
        );
        info!(
            "{tracking_label} - \
            trying to set existing user {user_id} \
            email={email} \
            with query='{query}'"
        );
        let stmt = conn.prepare(&query).await.unwrap();
        let _ = match conn.query(&stmt, &[]).await {
            Ok(query_result) => query_result,
            Err(e) => {
                let err_msg = format!("{e}");
                if err_msg.contains("duplicate key value violates") {
                    return Err(format!("{email} is already in use"));
                } else {
                    return Err(format!(
                        "failed updaing user.email={email}, user.verified \
                        with err={e}"
                    ));
                }
            }
        };
    }

    let query = match is_new_user {
        true => {
            format!(
                "INSERT INTO \
                    users_verified (\
                        user_id, \
                        token, \
                        email, \
                        state, \
                        exp_date) \
                VALUES (\
                    {user_id}, \
                    '{token}', \
                    '{email}', \
                    {user_verified_value}, \
                    '{verification_expiration_timestamp}');"
            )
        }
        false => {
            format!(
                "UPDATE \
                    users_verified \
                SET \
                    email = '{email}',
                    state = {user_verified_value}, \
                    token = '{token}', \
                    exp_date = '{verification_expiration_timestamp}', \
                    verify_date = NULL \
                WHERE \
                    users_verified.user_id = {user_id};"
            )
        }
    };
    info!(
        "{tracking_label} - \
        upserting user_verified {user_id} \
        new_user={is_new_user} \
        verified={user_verified_value} \
        email to {email} \
        with query='{query}'"
    );
    let stmt = conn.prepare(&query).await.unwrap();
    let _ = match conn.query(&stmt, &[]).await {
        Ok(query_result) => query_result,
        Err(e) => {
            let err_msg = format!("{e}");
            if err_msg.contains("duplicate key value violates") {
                return Err(format!("{email} is already in use"));
            } else {
                return Err(format!(
                    "failed updaing user.email={email}, user.verified \
                    with err={e}"
                ));
            }
        }
    };

    Ok(token)
}
