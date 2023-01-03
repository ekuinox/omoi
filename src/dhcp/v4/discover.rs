use std::net::Ipv4Addr;

use anyhow::{bail, ensure, Result};
use async_trait::async_trait;
use dhcproto::{v4, Encodable, Encoder};

use super::{Context, Handler, Request, Transactions};
use crate::{conf::OmoiConfig, db::Db};

pub struct DiscoverHandler;

pub struct OfferResponse {
    ip_addr: Ipv4Addr,
    broadcast_address: Ipv4Addr,
    subnet_mask: Ipv4Addr,
    routers: Vec<Ipv4Addr>,
    domain_name_servers: Vec<Ipv4Addr>,
    address_lease_time: u32,
}

#[async_trait]
impl Handler for DiscoverHandler {
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
        ensure!(message.opts().msg_type() != Some(v4::MessageType::Discover));

        let offer = Self::offer(message.xid(), message.chaddr(), &db, &config, transactions)?;

        let mut resp = v4::Message::default();

        resp.opts_mut()
            .insert(v4::DhcpOption::MessageType(v4::MessageType::Offer));
        resp.opts_mut()
            .insert(v4::DhcpOption::BroadcastAddr(offer.broadcast_address));
        resp.opts_mut()
            .insert(v4::DhcpOption::DomainNameServer(offer.domain_name_servers));
        resp.opts_mut()
            .insert(v4::DhcpOption::Router(offer.routers));
        resp.opts_mut()
            .insert(v4::DhcpOption::AddressLeaseTime(offer.address_lease_time));
        resp.opts_mut()
            .insert(v4::DhcpOption::SubnetMask(offer.subnet_mask));

        resp.set_secs(0)
            .set_ciaddr(0)
            .set_yiaddr(offer.ip_addr)
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

impl DiscoverHandler {
    fn offer(
        xid: u32,
        hardware_address: &[u8],
        db: &Db,
        config: &OmoiConfig,
        transactions: Transactions,
    ) -> Result<OfferResponse> {
        let Some(subnet) = config.dhcp4.subnets.first() else {
            bail!("subnets is empty");
        };
        let host = config
            .dhcp4
            .hosts
            .iter()
            .find(|host| host.hardware_ethernet.bytes().to_vec() == hardware_address);
        let ip_addr = match host {
            Some(host) => host.fixed_address,
            None => {
                let ip = db.leases_tree()?.suggest(
                    hardware_address,
                    subnet.range.0,
                    subnet.range.1,
                    transactions.offered_ipv4_addresses()?,
                )?;
                transactions.new_transaction(xid, ip)?;
                ip
            }
        };
        let resp = OfferResponse {
            ip_addr,
            broadcast_address: subnet.broadcast_address,
            subnet_mask: subnet.netmask,
            domain_name_servers: subnet.domain_name_servers.clone(),
            address_lease_time: subnet.address_lease_time,
            routers: subnet.routers.clone(),
        };

        Ok(resp)
    }
}
