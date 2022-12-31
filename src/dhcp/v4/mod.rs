use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use anyhow::{bail, Result};
use dhcproto::{
    v4::{self, Message, Opcode, OptionCode},
    Decodable, Decoder, Encodable, Encoder,
};
use tokio::net::UdpSocket;

pub const BUFFER_SIZE: usize = 1024;

fn decode(buffer: &[u8]) -> Result<Message> {
    let mut decoder = Decoder::new(buffer);
    let message = Message::decode(&mut decoder)?;
    Ok(message)
}

pub async fn handle_request(
    socket: Arc<UdpSocket>,
    buffer: Vec<u8>,
    addr: SocketAddr,
) -> Result<()> {
    dbg!(addr);
    let req = decode(&buffer)?;
    let Some(v4::DhcpOption::MessageType(req_type)) = req.opts().get(OptionCode::MessageType) else {
        bail!("Message type is not included.");
    };

    let mut opts = v4::DhcpOptions::new();
    if let Some(v4::DhcpOption::ParameterRequestList(params)) =
        req.opts().get(v4::OptionCode::ParameterRequestList)
    {
        for code in params {
            match code {
                v4::OptionCode::BroadcastAddr => {
                    opts.insert(v4::DhcpOption::BroadcastAddr(Ipv4Addr::new(
                        192, 168, 0, 255,
                    )));
                }
                code => {
                    dbg!(code);
                }
            }
        }
    }
    dbg!(req.chaddr(), req_type);

    if !req.chaddr().starts_with(&[0, 0, 0]) {
        bail!("not target");
    }

    match req_type {
        v4::MessageType::Discover => {
            opts.insert(v4::DhcpOption::MessageType(v4::MessageType::Offer));
        }
        _ => {
            bail!("unimplemented");
        }
    }

    let mut res = Message::default();
    res.set_opcode(Opcode::BootReply)
        .set_htype(req.htype())
        .set_hops(0)
        .set_secs(0)
        .set_ciaddr(0)
        .set_yiaddr(Ipv4Addr::new(192, 168, 0, 9))
        .set_flags(req.flags())
        .set_giaddr(req.giaddr())
        .set_chaddr(req.chaddr())
        .set_sname(b"hizake")
        .set_opts(opts);

    let mut buffer = Vec::with_capacity(1024);
    let mut encoder = Encoder::new(&mut buffer);
    res.encode(&mut encoder)?;

    socket.send_to(&buffer,  (Ipv4Addr::new(255, 255, 255, 255), addr.port())).await?;

    Ok(())
}

pub async fn serve() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:67").await?;
    socket.set_broadcast(true)?;
    let socket = Arc::new(socket);

    loop {
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let (_size, addr) = socket.recv_from(&mut buffer).await?;
        let socket = socket.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_request(socket, buffer, addr).await {
                eprintln!("{e}");
            }
        });
    }
}
