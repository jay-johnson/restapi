//! The CoreServices struct implements the hyper trait:
//! https://docs.rs/hyper/latest/hyper/service/trait.Service.html#
//! for serving HTTP traffic with a customized struct containing:
//! - the static server configuration (``config`` member)
//! - the bb8 postgres db threadpool (``db_pool`` member)
//! - the kafka publisher threadpool (``kafka_pool`` member)
//! - server tls information (``tls_info`` member)
//! - tracking fields for local and remote addresses
//!
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use hyper::service::Service;
use hyper::Body;
use hyper::Request;
use hyper::Response;

use kafka_threadpool::kafka_publisher::KafkaPublisher;

use crate::core::core_config::CoreConfig;
use crate::core::server::core_http_request::CoreHttpRequest;
use crate::handle_request::handle_request;

use crate::tls::tls_info::TlsInfo;

/// CoreServices
///
/// A date type containing the
/// hyper [`Request`](hyper::Request)
/// with the
/// [`CoreConfig`](crate::core::core_config::CoreConfig)
/// server statics,
/// tls information, kafka threadpool
/// [`KafkaPublisher`](kafka_threadpool::kafka_publisher::KafkaPublisher),
/// and postgres client db threadpool
/// ([`Pool`](bb8::Pool))
///
/// ## Core Config
///
/// [`CoreConfig`](crate::core::core_config::CoreConfig)
/// for static configuration values
///
/// ## bb8 Postgres Threadpool
///
/// bb8 thread pool ([`Pool`](bb8::Pool)) containing a
/// [`PostgresConnectionManager`](bb8_postgres::PostgresConnectionManager)
/// client that encrypts the traffic with a
/// [`MakeTlsConnector`](postgres_native_tls::MakeTlsConnector)
/// for db client tls encryption
///
/// ## Kafka Threadpool
///
/// The ``kafka_pool`` (
/// [`KafkaPublisher`](kafka_threadpool::kafka_publisher::KafkaPublisher))
/// supports HTTP requests that need to publish messages to the
/// environment variable-configured kafka cluster.
///
/// ## Socket Data
///
/// `local_addr` - server address
/// `remote_addr` - client address
///
/// ## TLS Info
///
/// [`TlsInfo`](crate::tls::tls_info::TlsInfo) object for tls verification
/// (this is not optional with the default configuration).
///
#[derive(Clone)]
pub struct CoreServices {
    pub config: CoreConfig,
    pub db_pool: Pool<PostgresConnectionManager<MakeTlsConnector>>,
    pub kafka_pool: KafkaPublisher,
    pub local_addr: std::net::SocketAddr,
    pub remote_addr: std::net::SocketAddr,
    pub tls_info: Option<TlsInfo>,
}

// Trait for hyper [`Service`](hyper::service::Service)
// <https://docs.rs/hyper/latest/hyper/service/trait.Service.html#>
impl Service<Request<Body>> for CoreServices {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<
        Box<
            dyn Future<
                    Output = std::result::Result<Self::Response, Self::Error>,
                > + Send,
        >,
    >;

    /// poll_ready
    ///
    /// please refer to the hyper docs:
    /// [`poll_ready`](hyper::service::Service::poll_ready)
    /// <https://docs.rs/hyper/latest/hyper/service/trait.Service.html#tymethod.poll_ready>
    ///
    /// # Arguments
    ///
    /// * `_cx` - [`Context`](std::task::Context)
    ///
    fn poll_ready(
        &mut self,
        _cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    /// call
    ///
    /// Wrap a received hyper [`Request`](hyper::Request) from
    /// the api server in a
    /// [`CoreHttpRequest`](crate::core::server::core_http_request::CoreHttpRequest)
    /// object and consume the
    /// [`CoreHttpRequest`](crate::core::server::core_http_request::CoreHttpRequest)
    /// with the
    /// [`handle_request`](crate::handle_request::handle_request)
    /// function
    ///
    /// Please refer to the hyper docs:
    /// [`call`](hyper::service::Service::call)
    /// <https://docs.rs/hyper/latest/hyper/service/trait.Service.html#tymethod.call>
    ///
    /// # Arguments
    ///
    /// * `req` - a Hyper [`Request`](hyper::Request)
    ///
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // build a task item containing everything
        // a request needs
        let data = CoreHttpRequest {
            config: self.config.clone(),
            db_pool: self.db_pool.clone(),
            kafka_pool: self.kafka_pool.clone(),
            local_addr: self.local_addr,
            remote_addr: self.remote_addr,
            tls_info: self.tls_info.clone(),
            request: req,
            response: Response::new("".into()),
        };
        // handle request
        Box::pin(handle_request(data))
    }
}
