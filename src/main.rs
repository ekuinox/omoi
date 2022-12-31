mod protocol;

use anyhow::Result;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = UdpSocket::bind("0.0.0.0:67").await?;
    listener.set_broadcast(true)?;

    loop {
        let mut buffer = vec![0u8; 1024];
        let (size, _addr) = listener.recv_from(&mut buffer).await?;
        let buffer = &buffer[..size];
        let message = protocol::decode(&buffer)?;
        dbg!(message);
    }
}
