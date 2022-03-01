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
    pub mode: String,
    pub socket_addr: std::net::SocketAddr,
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
            cert={} \
            key={} \
            ca={} \
            mode={}",
                self.enabled,
                self.cert_path,
                self.key_path,
                self.ca_path,
                self.mode);
        return true;
    }
}
