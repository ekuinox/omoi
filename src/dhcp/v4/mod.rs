mod lease;

use anyhow::{bail, Result};
use dhcproto::{
    v4::{self, Message, Opcode},
    Decodable, Decoder, Encodable, Encoder,
};
use mac_address::MacAddress;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::net::UdpSocket;

use crate::dhcp::v4::lease::LeaseRequest;

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

    // 後で消す
    if !req.chaddr().starts_with(&[0, 0, 0]) {
        bail!("not target");
    }

    let mut res = Message::default();
    match req_type {
        v4::MessageType::Discover => {
            opts.insert(v4::DhcpOption::MessageType(v4::MessageType::Offer));
            res.set_secs(0)
                .set_ciaddr(0)
                .set_yiaddr(req.yiaddr())
                .set_flags(req.flags())
                .set_giaddr(req.giaddr())
                .set_chaddr(req.chaddr());
        }
        v4::MessageType::Request => {
            let chaddr = req.chaddr();
            let resp = LeaseRequest::new(MacAddress::new([
                chaddr[0], chaddr[1], chaddr[2], chaddr[3], chaddr[4], chaddr[5],
            ]))
            .request();
            match resp {
                Ok(resp) => {
                    opts.insert(v4::DhcpOption::MessageType(v4::MessageType::Ack));
                    opts.insert(v4::DhcpOption::BroadcastAddr(resp.broadcast_address));
                    opts.insert(v4::DhcpOption::DomainNameServer(resp.domain_name_servers));
                    opts.insert(v4::DhcpOption::Router(resp.routers));
                    opts.insert(v4::DhcpOption::AddressLeaseTime(resp.address_lease_time));
                    res.set_secs(0)
                        .set_ciaddr(0)
                        .set_yiaddr(resp.ip_addr)
                        .set_flags(req.flags())
                        .set_giaddr(req.giaddr())
                        .set_chaddr(req.chaddr());
                }
                Err(e) => {
                    eprintln!("{e}");
                    opts.insert(v4::DhcpOption::MessageType(v4::MessageType::Nak));
                    res.set_secs(0)
                        .set_ciaddr(0)
                        .set_yiaddr(0)
                        .set_flags(req.flags())
                        .set_giaddr(req.giaddr())
                        .set_chaddr(req.chaddr());
                }
            }
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

fn destination(req: &Message, _res: &Message) -> (Ipv4Addr, bool) {
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
