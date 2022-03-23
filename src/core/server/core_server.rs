use std::sync::Arc;

use crate::pools::get_db_pool::get_db_pool;
use crate::tls::tls_info::TlsInfo;

use crate::core::core_config::CoreConfig;
use crate::core::server::connection_handler::ConnectionHandler;

/// core_server
///
/// Contains the server thread loop that starts everything
///
/// # Tasks
///
/// 1. Build the encrypted bb8 threadpool ([`Pool`](bb8::Pool))
/// 2. Build the [`TcpListener`](tokio::net::TcpListener) and bind it to
///    the api server address
/// 3. Create the [`Http`](hyper::server::conn::Http) server with
///    a thread-safe `Arc` for the verifying client tls connections using
///    a [`TlsInfo`](crate::tls::tls_info::TlsInfo) object
/// 4. Start the server `loop`
/// 5. Wait for a client connection `accept` is triggered on the server socket
/// 6. Clone all `Arc` objects to ensure thread-safety
/// 7. Create task future
/// 8. Start task future
/// 9. Determine if the client connection meets the tls requirements
/// 10. Extract client tls connection information
/// 11. Build a
///     [`ConnectionHandler`](crate::core::server::connection_handler::ConnectionHandler)
///     to wrap the [`CoreConfig`](crate::core::core_config::CoreConfig),
///     `bb8 threadpool for postrgres`, socket information (local and remote),
///     and tls client information
/// 12. Handle serving the client
///     connection using the [`handle_request`](crate::handle_request::handle_request)
///     function
///
/// # Arguments
///
/// * `config` - [`CoreConfig`](crate::core::core_config::CoreConfig) for static values
///
pub async fn core_server(
    config: &CoreConfig)
-> std::result::Result<String, hyper::Error>
{
    // 1
    let pool = get_db_pool(&config).await;
    // 2
    let listener = match tokio::net::TcpListener::bind(
            &config.api_config.socket_addr.unwrap()).await {
        Ok(v) => v,
        Err(e) => {
            let err_msg = format!("\
                Server startup failed - unable to \
                open server server_endpoint: {} with err='{e}' - stopping",
                config.api_config.server_endpoint);
            error!("{err_msg}");
            panic!("{err_msg}");
        }
    };
    let local_addr = listener.local_addr().unwrap();
    // 3
    let http = hyper::server::conn::Http::new();
    let acceptor = tokio_rustls::TlsAcceptor::from(
        Arc::new(
            config.api_config.server_config.clone()));

    // 4
    loop {
        // 5
        let (conn, remote_addr) = listener.accept().await.unwrap();
        // 6
        let acceptor = acceptor.clone();
        let http = http.clone();
        let cloned_config = config.clone();
        let cloned_pool = pool.clone();
        // 7
        let fut = async move {
            // 9 determine if the client connection meets the tls requirements
            match acceptor.accept(conn).await {
                Ok(stream) => {
                    // 10
                    let (_io, tls_connection) = stream.get_ref();

                    // 11
                    let handler = ConnectionHandler {
                        config: cloned_config,
                        db_pool: cloned_pool,
                        local_addr: local_addr,
                        remote_addr: remote_addr,
                        // tls is required
                        tls_info: Some(TlsInfo::from_tls_connection(tls_connection)),
                    };
                    // 12
                    if let Err(e) = http.serve_connection(stream, handler).await {
                        let err_msg = format!("{e}");
                        if ! err_msg.contains("connection error: not connected") {
                            error!("hyper server hit an internal error: {e}");
                        }
                    }
                },
                Err(e) => error!("hyper server tls error: {e}")
            }
        };
        // 8
        tokio::spawn(fut);
    }
}
