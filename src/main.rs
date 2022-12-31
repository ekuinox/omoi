mod protocol;

use anyhow::Result;
use bincode::Options;
use std::io::BufReader;
use tokio::net::UdpSocket;

use crate::protocol::DhcpMessageFormat;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = UdpSocket::bind("0.0.0.0:67").await?;
    listener.set_broadcast(true)?;

    let bincode = bincode::options()
        .with_big_endian()
        .allow_trailing_bytes()
        .with_fixint_encoding();

    loop {
        let mut buffer = vec![0u8; 1024];
        let (size, addr) = listener.recv_from(&mut buffer).await?;
        let buffer = &buffer[..size];
        dbg!(buffer);
        let reader = BufReader::new(buffer);
        let message: DhcpMessageFormat = bincode.deserialize_from(reader)?;
        dbg!(addr, message.options);
    }
}
