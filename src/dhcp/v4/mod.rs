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
    let Some(req_type) = req.opts().msg_type() else {
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

    // 後で消す
    if !req.chaddr().starts_with(&[0, 0, 0]) {
        bail!("not target");
    }

    let mut res = Message::default();
    match req_type {
        v4::MessageType::Discover => {
            opts.insert(v4::DhcpOption::MessageType(v4::MessageType::Offer));
            let yiaddr = if req.yiaddr().is_unspecified() {
                // todo
                Ipv4Addr::new(192, 168, 0, 1)
            } else {
                req.yiaddr()
            };
            res.set_secs(0)
                .set_ciaddr(0)
                .set_yiaddr(yiaddr)
                .set_flags(req.flags())
                .set_giaddr(req.giaddr())
                .set_chaddr(req.chaddr())
                .set_sname(b"hizake");
        }
        v4::MessageType::Request => {
            opts.insert(v4::DhcpOption::MessageType(v4::MessageType::Ack));
            let yiaddr = if req.yiaddr().is_unspecified() {
                // todo
                Ipv4Addr::new(192, 168, 0, 1)
            } else {
                req.yiaddr()
            };
            res.set_secs(0)
                .set_ciaddr(0)
                .set_yiaddr(yiaddr)
                .set_flags(req.flags())
                .set_giaddr(req.giaddr())
                .set_chaddr(req.chaddr())
                .set_sname(b"hizake");
        }
        ty => {
            bail!("unimplemented type={ty:?}");
        }
    }
    res.set_opcode(Opcode::BootReply)
        .set_htype(req.htype())
        .set_hops(0)
        .set_xid(req.xid())
        .set_opts(opts);

    let mut buffer = Vec::with_capacity(1024);
    let mut encoder = Encoder::new(&mut buffer);
    dbg!(&req, &res);
    res.encode(&mut encoder)?;

    let (dest, is_unicast) = destination(&req, &res);
    if is_unicast {
        todo!()
    }

    socket.send_to(&buffer, (dest, v4::CLIENT_PORT)).await?;

    Ok(())
}

fn destination(req: &Message, res: &Message) -> (Ipv4Addr, bool) {
    if req.ciaddr().is_unspecified() {
        return (req.ciaddr(), false);
    }
    todo!()
}

pub async fn serve() -> Result<()> {
    let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), v4::SERVER_PORT)).await?;
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
