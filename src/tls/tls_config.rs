use rustls::ServerConfig;

/// TlsConfig
///
/// Object containing a common tls configuration for a client
/// (the bb8 threadpool connected to postgres)
/// or a server (the hyper server threadpool listening to
/// requests)
///
#[derive(Clone)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: String,
    pub client_cert_path: String,
    pub client_key_path: String,
    pub client_ca_path: String,
    pub mode: String,
    pub socket_addr: Option<std::net::SocketAddr>,
    pub server_endpoint: String,
    // https://docs.rs/rustls/latest/rustls/struct.ServerConfig.html
    pub server_config: ServerConfig,
}

/// helper implementation method for debugging TlsConfig(s)
impl TlsConfig {

    /// show
    ///
    /// helper function for debugging filesystem
    /// paths vs expected environment variables
    ///
    pub fn show(&self) -> bool {
        println!("\
            enabled={} \
            server_endpoint={} \
            cert={} \
            key={} \
            ca={} \
            client_cert={} \
            client_key={} \
            client_ca={} \
            mode={}",
                self.enabled,
                self.server_endpoint,
                self.cert_path,
                self.key_path,
                self.ca_path,
                self.client_cert_path,
                self.client_key_path,
                self.client_ca_path,
                self.mode);
        return true;
    }
}
