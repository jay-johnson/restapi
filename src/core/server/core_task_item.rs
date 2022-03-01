use postgres_native_tls::MakeTlsConnector;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use hyper::Body;
use hyper::Request;
use hyper::Response;

use crate::core::core_config::CoreConfig;
use crate::tls::tls_info::TlsInfo;

/// CoreTaskItem
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
/// Everything a growing request needs!
///
pub struct CoreTaskItem {
    pub config: CoreConfig,
    pub db_pool: Pool<PostgresConnectionManager<MakeTlsConnector>>,
    pub local_addr: std::net::SocketAddr,
    pub remote_addr: std::net::SocketAddr,
    pub tls_info: Option<TlsInfo>,
    pub request: Request<Body>,
    pub response: Response<Body>,
}
