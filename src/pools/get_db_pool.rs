//! The ``get_db_pool`` function will start up the
//! bb8 postgres db threadpool based off environment variables
//!
use native_tls::Certificate as native_tls_cert;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use crate::core::core_config::CoreConfig;

/// get_db_pool
///
/// Build a bb8 threadpool ([`Pool](bb8::Pool)) providing a
/// [`PostgresConnectionManager`](bb8_postgres::PostgresConnectionManager)
/// client with tls encryption implemented using
/// [`MakeTlsConnector`](postgres_native_tls::MakeTlsConnector)
///
/// # Arguments
///
/// * `config` - [`CoreConfig`](crate::core::core_config::CoreConfig)
///
/// # Errors
///
/// The server will not start if the postgres db is not running
///
/// # Examples
///
/// ```rust
/// use restapi::core::core_config::build_core_config;
/// use restapi::pools::get_db_pool::get_db_pool;
/// let config = tokio_test::block_on(
///     build_core_config("test-get_db_pool")
/// ).unwrap();
/// let db_pool = tokio_test::block_on(
///     get_db_pool(&config)
/// );
/// ```
///
pub async fn get_db_pool(
    config: &CoreConfig,
) -> Pool<PostgresConnectionManager<MakeTlsConnector>> {
    let ca_bytes = std::fs::read(&config.db_config.ca_path).unwrap();
    let db_tls_ca = native_tls_cert::from_pem(&ca_bytes).unwrap();
    // use the certificate authority file
    let connector = TlsConnector::builder()
        .add_root_certificate(db_tls_ca)
        .build()
        .unwrap();
    let connector = MakeTlsConnector::new(connector);
    let db_conn_no_password = format!(
        "{}://{}:REDACTED@{}/{}?\
        sslmode=require",
        config.db_conn_type,
        config.db_username,
        config.db_address,
        config.db_name
    );
    let db_conn_str = format!(
        "{}://{}:{}@{}/{}?\
        sslmode=require",
        config.db_conn_type,
        config.db_username,
        config.db_password,
        config.db_address,
        config.db_name
    );
    info!(
        "connecting to postgres: {db_conn_no_password} \
        with db_tls_ca={}",
        config.db_config.ca_path
    );
    let pg_mgr =
        PostgresConnectionManager::new_from_stringlike(db_conn_str, connector)
            .unwrap();

    match Pool::builder().build(pg_mgr).await {
        Ok(pool) => pool,
        Err(e) => {
            panic!(
                "bb8 db threadpool hit an error '{e}' \
                connecting to {db_conn_no_password} \
                with db_tls_ca={}",
                config.db_config.ca_path
            )
        }
    }
}
