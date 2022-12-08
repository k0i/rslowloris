use anyhow::{bail, Result};
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
};

const MSG: &[u8] = b"X-a: 1\r";

// initialize a socket
pub fn init_socket<P: ToSocketAddrs>(to: P) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(to)
}

// write a message to a socket and return the next index
pub fn write_msg<T>(writer: &mut T) -> Result<()>
where
    T: Write + Read,
{
    if let Err(e) = writer.write_all(MSG) {
        bail!("Removing writer...: {}", e);
    }
    Ok(())
}
