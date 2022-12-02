use std::{env, io::BufRead};

use anyhow::{bail, Context, Result};

pub struct SlowlorisConfig {
    verbose: bool,
    http_only: bool,
    socket_count: usize,
    port: u16,
    url: String,
    proxies: Vec<String>,
}

impl Default for SlowlorisConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            http_only: false,
            socket_count: 1000,
            port: 443,
            url: String::new(),
            proxies: Vec::new(),
        }
    }
}

impl SlowlorisConfig {
    pub fn new(c: &seahorse::Context) -> Result<Self> {
        let mut conf = Self::default();
        // handle verbose flag
        if c.bool_flag("verbose") {
            env::set_var("RUST_LOG", "DEBUG");
        } else {
            env::set_var("RUST_LOG", "INFO");
        }
        conf.verbose = c.bool_flag("verbose");
        env_logger::init();

        // early return if no args
        if c.args.is_empty() {
            bail!("Please provide a url");
        }

        // handle http only flag
        if c.bool_flag("httponly") {
            conf.http_only = true;
        }
        // handle port flag
        if let Ok(port) = c.int_flag("port") {
            conf.port = port as u16;
        }
        // default port to 80 if http only
        if conf.http_only {
            conf.port = 80;
        }
        // handle socket count flag
        if let Ok(sock_cnt) = c.int_flag("socket") {
            conf.socket_count = sock_cnt as usize;
        }
        // handle url
        conf.url = format!("{}:{}", c.args[0].clone(), conf.port);
        // handle proxy file path
        if let Ok(proxy_file_path) = c.string_flag("proxy_file_path") {
            conf.proxies = Self::get_proxies_from_file(proxy_file_path)?;
        }
        Ok(conf)
    }
    fn get_proxies_from_file(path: String) -> Result<Vec<String>> {
        let file = std::fs::File::open(path).with_context(|| "Could not open proxy file")?;
        let reader = std::io::BufReader::new(file);
        let mut proxies = Vec::new();
        for line in reader.lines() {
            let line = line.with_context(|| "Could not parse line")?;
            proxies.push(line);
        }
        Ok(proxies)
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn socket_count(&self) -> usize {
        self.socket_count
    }
    pub fn proxies(&self) -> &Vec<String> {
        &self.proxies
    }
    pub fn http_only(&self) -> bool {
        self.http_only
    }
}
