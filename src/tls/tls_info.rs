//! Module containing the tls information struct and implementation
//! (``TlsInfo``) for the hyper serve to enable encryption in transit
//!
use rustls::ProtocolVersion;
use rustls::ServerConnection;
use rustls::SupportedCipherSuite;

/// TlsInfo
///
/// client tls attributes to verify the encryption
/// meets client and server requirements
/// to establish a secure link
///
#[derive(Default, Clone)]
pub struct TlsInfo {
    // server name indication
    // https://www.cloudflare.com/learning/ssl/what-is-sni/
    pub sni_hostname: Option<String>,
    // negotiation protocol
    // https://en.wikipedia.org/wiki/Application-Layer_Protocol_Negotiation
    pub alpn_protocol: Option<String>,
    // https://en.wikipedia.org/wiki/Cipher_suite
    pub ciphersuite: Option<SupportedCipherSuite>,
    // tls protocol version
    // https://en.wikipedia.org/wiki/Transport_Layer_Security#Secure_Data_Network_System
    pub version: Option<ProtocolVersion>,
}

/// tls info trait for a hyper Service
impl TlsInfo {
    /// from_tls_connection
    ///
    /// extract client tls information from the hyper [`Request`](hyper::Request) and
    /// create a `TlsInfo` object for tls verficication
    ///
    /// # Arguments
    ///
    /// * `conn` - [`ServerConnection`](rustls::ServerConnection) for extracting client tls information from the received hyper [`Request`](hyper::Request)
    ///
    pub fn from_tls_connection(conn: &ServerConnection) -> TlsInfo {
        TlsInfo {
            sni_hostname: conn.sni_hostname().map(|s| s.to_owned()),
            alpn_protocol: conn
                .alpn_protocol()
                .map(|s| String::from_utf8_lossy(s).into_owned()),
            ciphersuite: conn.negotiated_cipher_suite(),
            version: conn.protocol_version(),
        }
    }
}
