//! When the hyper server receives an HTTP request it is assigned to a
//! single worker thread and the HTTP request is stored in a:
//! [`CoreHttpRequest`](crate::core::server::core_http_request::CoreHttpRequest)
//! object for the single hyper worker thread to serve the request.
//! To do this, the ``CoreHttpRequest`` must store everything the
//! worker thread will need to serve the HTTP request including:
//! - the static server configuration in the member field::
//! [`config: CoreConfig`](crate::core::core_config::CoreConfig)
//! - the postgres bb8 db threadpool in the member field:
//! [`db_pool: Pool<PostgresConnectionManager<MakeTlsConnector>>`](bb8::Pool)
//! - the kafka threadpool's
//! [`KafkaPublisher`](kafka_threadpool::KafkaPublisher)
//! in the member field:
//! [`kafka_pool: KafkaPublisher`](kafka_threadpool::KafkaPublisher)
//! - the HTTP request in the member field:
//! [`request: Request<Body>`](hyper::Request)
//! - the HTTP response in the member field:
//! [`response: Response`](hyper::Response)
//!
use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use hyper::Body;
use hyper::Request;
use hyper::Response;

use kafka_threadpool::kafka_publisher::KafkaPublisher;

use crate::core::core_config::CoreConfig;
use crate::tls::tls_info::TlsInfo;

/// CoreHttpRequest
///
/// Wrapper for building an internal object that owns both
/// a hyper [`Request`](hyper::Request) and a
/// [`Response`](hyper::Response). It also owns all cloned
/// statics (config is a
/// [`CoreConfig`](crate::core::core_config::CoreConfig)
/// ),
/// tls information (tls_info is a
/// [`TlsInfo`](crate::tls::tls_info::TlsInfo)
/// ), and db_pool is a [`Pool`](bb8::Pool) reference to the
/// postgres client db threadpool.
///
/// If the environment variable ``KAFKA_ENABLED=1`` then
/// the ``kafka_pool`` allows for each HTTP request to
/// publish messages to the configured kafka cluster.
/// Please see the
/// [kafka_threadpool docs](https://docs.rs/kafka-threadpool/latest/kafka_threadpool/)
/// for more information on how to configure the
/// kafka publisher threadpool.
///
/// Everything a growing request needs!
///
pub struct CoreHttpRequest {
    pub config: CoreConfig,
    pub db_pool: Pool<PostgresConnectionManager<MakeTlsConnector>>,
    pub kafka_pool: KafkaPublisher,
    pub local_addr: std::net::SocketAddr,
    pub remote_addr: std::net::SocketAddr,
    pub tls_info: Option<TlsInfo>,
    pub request: Request<Body>,
    pub response: Response<Body>,
}
