use postgres_native_tls::MakeTlsConnector;

use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;

use serde::Deserialize;
use serde::Serialize;

/// ModelUser
///
/// Representation of the users table in the db
///
/// # DB table
///
/// `users`
///
/// # Arguments
///
/// * `id` - `i32` - user id
/// * `email` - `String` - email address
/// * `password` - `String` - salted password
/// * `state` - `i32` - is the user
///   active (`0`) or inactive (`1`)
/// * `verified` - `i32` - is the user email
///   unverified (`0`) or verified (`1`)
/// * `role` - `String` - user's role
///
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelUser {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub state: i32,
    pub verified: i32,
    pub role: String,
}

/// get_user_by_id
///
/// Get a user from the database by `user_id`
///
/// # Arguments
///
/// * `tracking_label` - `&str` - caller logging label
/// * `id` - `i32` - user id
/// * `conn` - [`PooledConnection`](bb8::PooledConnection) -
///   an established db connection from the
///   postgres client db threadpool
///
/// # Returns
///
/// ## get_user_otp on Success Returns
///
/// [`ModelUser`](crate::requests::models::user)
///
/// # Errors
///
/// Various `Err(String)` can be returned depending
/// on what breaks
///
pub async fn get_user_by_id(
    tracking_label: &str,
    id: i32,
    conn: &PooledConnection<'_, PostgresConnectionManager<MakeTlsConnector>>,
) -> Result<ModelUser, String> {
    // find all user by email and an active state where state == 0
    let query = format!(
        "SELECT \
            users.id, \
            users.email, \
            users.password, \
            users.state, \
            users.verified, \
            users.role \
        FROM \
            users \
        WHERE \
            users.id = {id} \
        LIMIT 1;"
    );
    let stmt = conn.prepare(&query).await.unwrap();
    match conn.query(&stmt, &[]).await {
        Ok(query_result) => {
            // get just the first element
            if let Some(row) = query_result.first() {
                let id: i32 = row.try_get("id").unwrap();
                let email: String = row.try_get("email").unwrap();
                let password: String = row.try_get("email").unwrap();
                let state: i32 = row.try_get("state").unwrap();
                let verified: i32 = row.try_get("verified").unwrap();
                let role: String = row.try_get("role").unwrap();
                return Ok(ModelUser {
                    id,
                    email,
                    password,
                    state,
                    verified,
                    role,
                });
            }
            Err(format!(
                "{tracking_label} - \
                failed to find any user with id={id}"
            ))
        }
        Err(e) => Err(format!(
            "{tracking_label} - \
                failed to find user by id={id} \
                with err='{e}'"
        )),
    }
}
