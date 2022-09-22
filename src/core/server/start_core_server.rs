//! Before the Rest API server can start serving HTTP requests
//! it starts up all configured threadpools and creates
//! tls encryption in transit objects. These are stored in
//! the [`CoreServices`](crate::core::server::core_services::CoreServices) struct
//! which is cloned and passed to
//! each tokio-spawned worker thread when a new HTTP request is received
//!
use std::sync::Arc;

use kafka_threadpool::kafka_publisher::KafkaPublisher;
use kafka_threadpool::start_threadpool::start_threadpool;

use crate::pools::get_db_pool::get_db_pool;
use crate::tls::tls_info::TlsInfo;

use crate::core::core_config::CoreConfig;
use crate::core::server::core_services::CoreServices;

/// start_core_server
///
/// Contains the server thread loop that starts everything
///
/// # Tasks
///
/// 1. Start threadpools based off the ``CoreConfig``
///    - Build the encrypted bb8 threadpool ([`Pool`](bb8::Pool))
///    - Build the encrypted kafka threadpool
///      ([`KafkaPublisher`](kafka_threadpool::KafkaPublisher))
/// 1. Build the [`TcpListener`](tokio::net::TcpListener) and bind it to
///    the api server address
/// 1. Create the [`Http`](hyper::server::conn::Http) server with
///    a thread-safe `Arc` for the verifying client tls connections using
///    a [`TlsInfo`](crate::tls::tls_info::TlsInfo) object
/// 1. Start the server `loop`
/// 1. Wait for a client connection `accept` is triggered on the server socket
/// 1. Clone all `Arc` objects to ensure thread-safety
/// 1. Create task future
/// 1. Start task future
/// 1. Determine if the client connection meets the tls requirements
/// 1. Extract client tls connection information
/// 1. Build a
///    [`CoreServices`](crate::core::server::core_services::CoreServices)
///    to wrap the [`CoreConfig`](crate::core::core_config::CoreConfig),
///    `bb8 threadpool for postrgres`, socket information (local and remote),
///    and tls client information
/// 1. Handle serving the client
///    connection using the [`handle_request`](crate::handle_request::handle_request)
///    function
///
/// # Arguments
///
/// * `config` -
/// [`CoreConfig`](crate::core::core_config::CoreConfig)
/// for static values read from environment variables
///
pub async fn start_core_server(
    config: &CoreConfig,
) -> std::result::Result<String, hyper::Error> {
    // 1 - start threadpools
    let db_pool = get_db_pool(config).await;
    let kafka_pool: KafkaPublisher =
        start_threadpool(Some(&config.label)).await;
    // 2
    let listener = match tokio::net::TcpListener::bind(
        &config.api_config.socket_addr.unwrap(),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            let err_msg = format!(
                "Server startup failed - unable to \
                open server server_endpoint: {} with err='{e}' - stopping",
                config.api_config.server_endpoint
            );
            error!("{err_msg}");
            panic!("{err_msg}");
        }
    };
    let local_addr = listener.local_addr().unwrap();
    // 3
    let http = hyper::server::conn::Http::new();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(
        config.api_config.server_config.clone(),
    ));

    // 4
    loop {
        // 5
        let (conn, remote_addr) = listener.accept().await.unwrap();
        // 6
        let acceptor = acceptor.clone();
        let http = http.clone();
        let cloned_config = config.clone();
        let cloned_db_pool = db_pool.clone();
        let cloned_kafka_pool = kafka_pool.clone();
        // 7
        let fut = async move {
            // 9 determine if the client connection meets the tls requirements
            match acceptor.accept(conn).await {
                Ok(stream) => {
                    // 10
                    let (_io, tls_connection) = stream.get_ref();

                    // 11
                    let supported_services = CoreServices {
                        config: cloned_config,
                        db_pool: cloned_db_pool,
                        kafka_pool: cloned_kafka_pool,
                        local_addr,
                        remote_addr,
                        // tls is required
                        tls_info: Some(TlsInfo::from_tls_connection(
                            tls_connection,
                        )),
                    };
                    // 12
                    if let Err(e) =
                        http.serve_connection(stream, supported_services).await
                    {
                        let err_msg = format!("{e}");
                        if !err_msg.contains("connection error: not connected")
                            && !err_msg
                                .contains("connection error: connection reset")
                        {
                            trace!("hyper server hit an internal error: {e}");
                        }
                    }
                }
                Err(e) => error!("hyper server tls error: {e}"),
            }
        };
        // 8
        tokio::spawn(fut);
    }
}
