use std::net::SocketAddr;

use anyhow::{Result, bail};
use dhcproto::{v4::{Message, OptionCode}, Decodable, Decoder};
use tokio::net::UdpSocket;

pub const BUFFER_SIZE: usize = 1024;

fn decode(buffer: &[u8]) -> Result<Message> {
    let mut decoder = Decoder::new(buffer);
    let message = Message::decode(&mut decoder)?;
    Ok(message)
}

pub async fn handle_request(buffer: Vec<u8>, _addr: SocketAddr) -> Result<()> {
    let message = decode(&buffer)?;
    let Some(message_type) = message.opts().get(OptionCode::MessageType) else {
        bail!("Message type is not included.");
    };
    dbg!(message.chaddr(), message_type);
    Ok(())
}

pub async fn serve() -> Result<()> {
    let listener = UdpSocket::bind("0.0.0.0:67").await?;
    listener.set_broadcast(true)?;

    loop {
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let (_size, addr) = listener.recv_from(&mut buffer).await?;
        tokio::spawn(handle_request(buffer, addr));
    }
}
