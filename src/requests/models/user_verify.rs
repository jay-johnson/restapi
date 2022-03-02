use postgres_native_tls::MakeTlsConnector;

use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;

use serde::Serialize;
use serde::Deserialize;

/// ModelUserVerify
///
/// Representation of the user's email verification
/// record in the db
///
/// # DB table
///
/// `users_verify`
///
/// # Arguments
///
/// * `id` - `i32` - verification db record id
/// * `user_id` - `i32` - user id
/// * `token` - `String` - verification token
/// * `email` - `String` - user's email address
/// * `state` - `i32` - is the user's email
///   verified (`1`) or not verified (`0` default)
/// * `exp_date_utc` - [`chrono::DateTime](chrono::Datetime)
///   when does the user's email verification token
///   expire in `Utc`
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelUserVerify {
    pub id: i32, 
    pub user_id: i32,
    pub token: String,
    pub email: String,
    pub state: i32,
    pub exp_date_utc: chrono::DateTime<chrono::Utc>,
}

/// get_user_verify_by_user_id
///
/// Get the user's email verification record from
/// the database.
///
/// # Arguments
///
/// * `tracking_label` - `&str` - caller logging label
/// * `user_id` - `i32` - user id
/// * `conn` - [`PooledConnection`](bb8::PooledConnection) -
///   an established db connection from the
///   postgres client db threadpool
///
/// # Returns
///
/// ## get_user_otp on Success Returns
///
/// [`ModelUserVerify`](crate::requests::models::user_verify)
///
/// # Errors
///
/// Various `Err(String)` can be returned depending
/// on what breaks
///
pub async fn get_user_verify_by_user_id(
    tracking_label: &str,
    user_id: i32,
    conn: &PooledConnection<'_, PostgresConnectionManager<MakeTlsConnector>>)
-> Result<ModelUserVerify, String>
{
    // find all user by email and an active state where state == 0
    let query = format!("\
        SELECT \
            users_verified.id, \
            users_verified.user_id, \
            users_verified.token, \
            users_verified.email, \
            users_verified.state, \
            users_verified.exp_date, \
            users_verified.created_at, \
            users_verified.verify_date, \
            users_verified.updated_at \
        FROM \
            users_verified \
        WHERE \
            users_verified.user_id = {user_id} \
        LIMIT 1;");
    // println!("{}", query);
    let stmt = conn.prepare(&query).await.unwrap();
    match conn.query(&stmt, &[]).await {
        Ok(query_result) => {
            for row in query_result.iter() {
                let id: i32 = row.try_get("id").unwrap();
                let user_id: i32 = row.try_get("user_id").unwrap();
                let token: String = row.try_get("token").unwrap();
                let email: String = row.try_get("email").unwrap();
                let state: i32 = row.try_get("state").unwrap();
                let exp_date_utc: chrono::DateTime<chrono::Utc> = 
                    row.try_get("exp_date").unwrap();
                return Ok(
                    ModelUserVerify {
                        id, 
                        user_id,
                        token,
                        email,
                        state,
                        exp_date_utc,
                    });
            }
            return Err(format!("\
                {tracking_label} - \
                failed to find any user verify with user_id={user_id}"));
        },
        Err(e) => {
            return Err(format!("\
                {tracking_label} - \
                failed to find user verify by user_id={user_id} \
                with err='{e}'"));
        }
    }
}
