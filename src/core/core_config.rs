use crate::tls::tls_config::TlsConfig;
use crate::tls::get_tls_config::get_tls_config;

/// CoreConfig
///
/// The server configuration struct for connectivity
/// and commonly-accessed statics (password salt,
/// jwt encoding/decoding keys, tls configurations
/// using [`TlsConfig`](crate::tls::tls_config::TlsConfig)
/// for the api and postgres threadpools).
///
/// # Supported Environment Variables
///
/// Configure the server configuration `CoreConfig`
/// with the environment variables and defaults
///
/// ## Server - Api Threadpool
///
/// ### Change the server listening address and port
///
/// ```bash
/// export API_ENDPOINT="0.0.0.0:3000"
/// ```
///
/// ## Server - Postgres Threadpool
///
/// ### Change the postgres database address and port
///
/// ```bash
/// export POSTGRES_ENDPOINT="0.0.0.0:5432"
/// ```
///
/// ### Change the postgres user credentials
///
/// ```bash
/// export POSTGRES_USERNAME="postgres"
/// export POSTGRES_PASSWORD="postgres"
/// ```
///
/// ### Change the postgres database
///
/// ```bash
/// export SERVER_DB_NODE_NAME="mydb"
/// ```
///
/// ### Change the user password salt for argon2 password hashing
///
/// ```bash
/// export SERVER_PASSWORD_SALT="PLEASE_CHANGE_ME"
/// ```
///
/// ## JWT using the `jsonwebtokens` crate and encrypted using `Algorithm::ES256` algorithm
///
/// ### Change jwt private key
///
/// ```bash
/// export TOKEN_ALGO_PRIVATE_KEY="path/private-key-pkcs8.pem"
/// ```
///
/// ### Change jwt public key
///
/// ```bash
/// export TOKEN_ALGO_PUBLIC_KEY="path/public-key.pem"
/// ```
///
/// ## Tls Environment Variables
///
/// ### Change the `API Server` tls certificate authority, server key and cert
///
/// ```bash
/// export API_TLS_CA="path/api-ca.pem"
/// export API_TLS_KEY="path/api.key"
/// export API_TLS_CERT="path/api.crt"
/// ```
///
/// ### Change the `Postgres` tls certificate authority
///
/// ```bash
/// export DB_TLS_CA="path/api-ca.pem"
/// ```
///
/// ## Logging
///
/// ### Set the server name for the logs
///
/// ```bash
/// export SERVER_NAME_LABEL="my-server"
/// ```
///
/// ## Debug
///
/// At startup, print a curl connectivity command
/// and an openssl ssl verification command
/// for postgres
///
/// ```bash
/// export DEBUG="1"
/// ```
///
#[derive(Clone)]
pub struct CoreConfig {
    pub label: String,
    pub server_address: String,
    pub server_password_salt: Vec<u8>,
    pub api_config: TlsConfig,
    pub db_conn_type: String,
    pub db_username: String,
    pub db_password: String,
    pub db_address: String,
    pub db_name: String,
    pub db_config: TlsConfig,
    pub encoding_key_bytes: Vec<u8>,
    pub decoding_key_bytes: Vec<u8>,
    // more shared Send/Sync objects can go here
}

/// build_core_config
///
/// Build a [`CoreConfig`](crate::core::core_config::CoreConfig)
/// from environment variables and files on disk.
///
/// # Arguments
///
/// * `label` - logging label
///
pub async fn build_core_config(
    label: &str)
-> Result<CoreConfig, String>
{
    let tracking_label = std::env::var("SERVER_NAME_LABEL")
        .unwrap_or(format!("{label}"));
    let api_name = std::env::var("SERVER_NAME_API")
        .unwrap_or(format!("api"));
    let api_address = std::env::var(format!("{api_name}_ENDPOINT").to_uppercase())
        .unwrap_or(format!("0.0.0.0:3000"));

    let api_tls_mode = "tls";
    let db_cert_name = std::env::var("SERVER_DB_NODE_NAME")
        .unwrap_or(format!("postgres"));
    let db_conn_type = std::env::var(format!("{db_cert_name}_DB_CONN_TYPE").to_uppercase())
        .unwrap_or(format!("postgresql"));
    let db_address = std::env::var(format!("{db_cert_name}_ENDPOINT").to_uppercase())
        .unwrap_or(format!("0.0.0.0:5432"));
    let db_username = std::env::var(format!("{db_cert_name}_USERNAME").to_uppercase())
        .unwrap_or(format!("datawriter"));
    let db_password = std::env::var(format!("{db_cert_name}_PASSWORD").to_uppercase())
        .unwrap_or(format!("123321"));
    let db_name = std::env::var("SERVER_DB_NODE_NAME")
        .unwrap_or(format!("mydb"));
    let db_tls_mode = "require";
    let server_password_salt = std::env::var("SERVER_PASSWORD_SALT")
        .unwrap_or(String::from("PLEASE_CHANGE_ME"));

    let pki_dir_jwt = std::env::var("SERVER_PKI_DIR_JWT")
        .unwrap_or(format!("./jwt"));
    let token_private_key_path = std::env::var("TOKEN_ALGO_PRIVATE_KEY")
        .unwrap_or(format!("{pki_dir_jwt}/private-key-pkcs8.pem"));
    let token_public_key_path = std::env::var("TOKEN_ALGO_PUBLIC_KEY")
        .unwrap_or(format!("{pki_dir_jwt}/public-key.pem"));

    let token_private_key_bytes = std::fs::read_to_string(&token_private_key_path)
        .unwrap()
        .into_bytes();
    let token_public_key_bytes = std::fs::read_to_string(&token_public_key_path)
        .unwrap()
        .into_bytes();

    let api_config = match get_tls_config(
            &tracking_label,
            &api_name,
            &api_address,
            &api_tls_mode).await {
        Ok(api_config) => api_config,
        Err(err_msg) => {
            panic!("\
                {tracking_label} - \
                failed to build {api_name} tls config with err='{err_msg}'");
        }
    };

    let db_config = match get_tls_config(
            &tracking_label,
            &db_cert_name,
            &db_address,
            &db_tls_mode).await {
        Ok(db_config) => db_config,
        Err(err_msg) => {
            panic!("\
                {label} - \
                failed to build {db_cert_name} tls config with err='{err_msg}'");
        }
    };

    if ! db_config.enabled {
        let err_msg = format!("\
            {tracking_label} - invalid tls for the db - stopping");
        error!("{err_msg}");
        return Err(err_msg);
    }

    // config object
    let config = CoreConfig {
        label: tracking_label,
        server_address: api_address,
        server_password_salt: server_password_salt.as_bytes().to_vec(),
        db_conn_type: db_conn_type,
        db_username: db_username,
        db_password: db_password,
        db_address: db_address,
        db_name: db_name,
        api_config: api_config,
        db_config: db_config,
        encoding_key_bytes: token_private_key_bytes.clone(),
        decoding_key_bytes: token_public_key_bytes.clone(),
    };

    if std::env::var("DEBUG").unwrap_or(format!("0")) == format!("1") {
        info!("\
            {label} - {api_name} listening on {}\n\
            test the api with:\n\
            \n\
            curl -iivv \
            --cacert {} \
            --cert {} \
            --key {} \
            \"https://{}\"\n\
            \n\
            test the db with:\n\
            openssl s_client -connect {} -starttls postgres\n\
            \n\
            token:\n\
            - private key: {token_private_key_path}\n\
            - public key: {token_public_key_path}\n\
            \n",
                config.server_address,
                config.api_config.ca_path,
                config.api_config.cert_path,
                config.api_config.key_path,
                config.server_address,
                config.db_address);
    }

    return Ok(config);
}
