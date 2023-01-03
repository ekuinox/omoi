use std::ops::Add;

use anyhow::{bail, ensure, Result};
use async_trait::async_trait;
use chrono::{Duration, Local};
use dhcproto::{v4, Encodable, Encoder};

use super::{Context, Handler, Request};

pub struct RequestHandler;

#[async_trait]
impl Handler for RequestHandler {
    async fn handle(
        &self,
        Request {
            context:
                Context {
                    db,
                    config,
                    transactions,
                    socket,
                },
            message,
        }: Request,
    ) -> Result<()> {
        ensure!(message.opts().msg_type() != Some(v4::MessageType::Request));
        let Some(subnet) = config.dhcp4.subnets.first() else {
            bail!("subnets is empty");
        };

        let ip_addr = match transactions.remove(message.xid()) {
            Ok(transaction) => transaction.offered_ipv4_addr,
            Err(_) => {
                let ip = db.leases_tree()?.suggest(
                    message.chaddr(),
                    subnet.range.0,
                    subnet.range.1,
                    transactions.offered_ipv4_addresses()?,
                )?;
                ip
            }
        };
        db.leases_tree()?.acquire(
            message.chaddr().to_vec(),
            ip_addr,
            Local::now().add(Duration::seconds(subnet.address_lease_time.into())),
        )?;

        let mut resp = v4::Message::default();

        resp.opts_mut()
            .insert(v4::DhcpOption::MessageType(v4::MessageType::Ack));
        resp.opts_mut()
            .insert(v4::DhcpOption::BroadcastAddr(subnet.broadcast_address));
        resp.opts_mut().insert(v4::DhcpOption::DomainNameServer(
            subnet.domain_name_servers.clone(),
        ));
        resp.opts_mut()
            .insert(v4::DhcpOption::Router(subnet.routers.clone()));
        resp.opts_mut()
            .insert(v4::DhcpOption::AddressLeaseTime(subnet.address_lease_time));
        resp.opts_mut()
            .insert(v4::DhcpOption::SubnetMask(subnet.netmask));

        resp.set_secs(0)
            .set_ciaddr(0)
            .set_yiaddr(ip_addr)
            .set_flags(message.flags())
            .set_giaddr(message.giaddr())
            .set_chaddr(message.chaddr())
            .set_opcode(v4::Opcode::BootReply)
            .set_htype(message.htype())
            .set_hops(0)
            .set_xid(message.xid());

        let mut buffer = Vec::with_capacity(1024);
        let mut encoder = Encoder::new(&mut buffer);
        resp.encode(&mut encoder)?;

        let dest = if message.ciaddr().is_unspecified() {
            message.ciaddr()
        } else {
            todo!()
        };

        socket.send_to(&buffer, (dest, v4::CLIENT_PORT)).await?;

        Ok(())
    }
}
