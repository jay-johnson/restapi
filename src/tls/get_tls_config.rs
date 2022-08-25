use rustls::Certificate;
use rustls::PrivateKey;
use rustls::ServerConfig;

use crate::tls::tls_config;

/// get_tls_config
///
/// Build a [`TlsConfig`](crate::tls::tls_config) for hosting
/// an encrypted endpoint.
///
/// # Usage
///
/// ## Environment variables with default values (bash):
///
/// ### Change the API server tls certificate authority, server certificate and key
///
/// ```bash
/// export API_TLS_CA="${API_TLS_DIR}/api-ca.pem"
/// export API_TLS_KEY="${API_TLS_DIR}/api.key"
/// export API_TLS_CERT="${API_TLS_DIR}/api.crt"
/// ```
///
/// ```bash
/// export DB_TLS_CA="${DB_TLS_DIR}/api-ca.pem"
/// export DB_TLS_KEY="${DB_TLS_DIR}/api.key"
/// export DB_TLS_CERT="${DB_TLS_DIR}/api.crt"
/// ```
///
/// # Arguments
///
/// * `tracking_label` - &str - label from caller function
/// * `app_name` - &str - directory name for tls assets
/// * `server_address` - &str - address to host the server's
///   listening port with format: IP_ADDRESS:PORT
/// * `mode` - `tls` for api's and `require` for postgres
///
/// # Examples
///
/// ```rust
/// use restapi::tls::tls_config::TlsConfig;
/// use restapi::tls::get_tls_config::get_tls_config;
///
/// let future_val = async {
///     let label = std::env::var("SERVER_NAME_LABEL")
///         .unwrap_or_else(|_| "get_tls_config-example".to_string());
///     let api_name = std::env::var("SERVER_NAME_API")
///         .unwrap_or_else(|_| "api".to_string());
///     let api_address = std::env::var(format!("{api_name}_ENDPOINT").to_uppercase())
///         .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
///     let api_tls_mode = "tls";
///
///     let api_config = match get_tls_config(
///             &label,
///             &api_name,
///             &api_address,
///             &api_tls_mode).await {
///         Ok(api_config) => api_config,
///         Err(err_msg) => {
///             panic!("failed to build {api_name} tls config with err='{err_msg}'");
///         }
///     };
///     api_config
/// };
/// // https://stackoverflow.com/questions/64568390/rust-doc-test-with-async-function-tokio-test
/// let cnf = tokio_test::block_on(future_val);
/// assert_eq!(cnf.enabled, true);
/// ```
///
pub async fn get_tls_config(
    tracking_label: &str,
    app_name: &str,
    server_address: &str,
    mode: &str,
) -> Result<tls_config::TlsConfig, String> {
    let tls_dir = std::env::var(format!("{app_name}_TLS_DIR"))
        .unwrap_or_else(|_| format!("./certs/tls/{app_name}"));
    let tls_ca = std::env::var(format!("{app_name}_TLS_CA"))
        .unwrap_or_else(|_| format!("{tls_dir}/{app_name}-ca.pem"));
    let tls_key = std::env::var(format!("{app_name}_TLS_KEY"))
        .unwrap_or_else(|_| format!("{tls_dir}/{app_name}.key"));
    let tls_cert = std::env::var(format!("{app_name}_TLS_CERT"))
        .unwrap_or_else(|_| format!("{tls_dir}/{app_name}.crt"));

    let mut tls_enabled = false;
    if !&tls_ca.is_empty() && !&tls_key.is_empty() && !&tls_cert.is_empty() {
        tls_enabled = true;
    }

    info!(
        "{tracking_label} - start \
        tls={tls_enabled} \
        ca={tls_ca} \
        key={tls_key} \
        cert={tls_cert}"
    );

    if std::fs::metadata(&tls_ca).is_err() {
        let err_msg = format!(
            "{tracking_label} - \
            failed to find {}_TLS_CA={tls_ca}",
            app_name.to_uppercase()
        );
        error!("{err_msg}");
        tls_enabled = false;
    }

    if std::fs::metadata(&tls_key).is_err() {
        let err_msg = format!(
            "{tracking_label} - \
            failed to find {}_TLS_KEY={tls_key}",
            app_name.to_uppercase()
        );
        error!("{err_msg}");
        tls_enabled = false;
    }

    if std::fs::metadata(&tls_cert).is_err() {
        let err_msg = format!(
            "{tracking_label} - \
            failed to find {}_TLS_CERT={tls_cert}",
            app_name.to_uppercase()
        );
        error!("{err_msg}");
        tls_enabled = false;
    }

    // load api certificates
    let cert_pem = std::fs::read(&*tls_cert).unwrap();
    let key_pem = std::fs::read(&*tls_key).unwrap();

    let server_config = {
        let certs: Vec<Certificate> = rustls_pemfile::certs(&mut &*cert_pem)
            .map(|mut certs| certs.drain(..).map(Certificate).collect())
            .unwrap();

        let mut keys: Vec<PrivateKey> =
            rustls_pemfile::pkcs8_private_keys(&mut &*key_pem)
                .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
                .unwrap();

        let mut server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, keys.remove(0))
            .unwrap();

        server_config.alpn_protocols =
            vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        server_config
    };

    Ok(tls_config::TlsConfig {
        enabled: tls_enabled,
        cert_path: tls_cert,
        key_path: tls_key,
        ca_path: tls_ca,
        // mtls client tls assets
        client_cert_path: "".to_string(),
        client_key_path: "".to_string(),
        client_ca_path: "".to_string(),
        mode: mode.to_string(),
        socket_addr: match server_address.parse::<std::net::SocketAddr>() {
            Ok(sa) => Some(sa),
            Err(_) => None,
        },
        server_endpoint: server_address.to_string(),
        server_config,
    })
}
