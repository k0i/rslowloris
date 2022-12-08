use std::{net::ToSocketAddrs, sync::Arc, time::SystemTime};

use rustls::{
    client::{ServerCertVerified, ServerCertVerifier},
    Certificate, ClientConnection, ServerName,
};

// forgo TLS certificate verification
struct NoCertificateVerification {}
impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

// initialize TLS configuration
pub fn init_tls<P: ToSocketAddrs>(ip: P) -> ClientConnection {
    let root_certs = rustls::RootCertStore::empty();
    let mut config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certs)
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoCertificateVerification {}));
    let arc_config = Arc::new(config);
    let socket_addr = ip
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("Failed to get socket address");
    ClientConnection::new(arc_config, ServerName::IpAddress(socket_addr.ip()))
        .expect("Failed to create TLS client")
}
