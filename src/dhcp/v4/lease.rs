use std::{net::Ipv4Addr, ops::Add};

use crate::{
    conf::{Dhcp4SubnetConfig, OMOI_CONFIG},
    db::{Db, Leased4Table},
};
use anyhow::{bail, Result};
use chrono::{Duration, Local};
use ipnet::Ipv4AddrRange;
use mac_address::MacAddress;

pub struct LeaseRequest {
    mac_address: MacAddress,
}

pub struct LeasedResponse {
    pub mac_address: MacAddress,
    pub ip_addr: Ipv4Addr,
    pub broadcast_address: Ipv4Addr,
    pub subnet_mask: Ipv4Addr,
    pub routers: Vec<Ipv4Addr>,
    pub domain_name_servers: Vec<Ipv4Addr>,
    pub address_lease_time: u32,
}

impl LeaseRequest {
    pub fn new(mac_address: MacAddress) -> LeaseRequest {
        LeaseRequest { mac_address }
    }

    pub fn request(self) -> Result<LeasedResponse> {
        let Some(subnet) = OMOI_CONFIG.dhcp4.subnets.first() else {
            bail!("subnets is empty");
        };
        let host = OMOI_CONFIG
            .dhcp4
            .hosts
            .iter()
            .find(|host| host.hardware_ethernet == self.mac_address);
        let ip_addr = match host {
            Some(host) => host.fixed_address,
            None => get_new_ip(&subnet, self.mac_address)?,
        };
        let resp = LeasedResponse {
            mac_address: self.mac_address,
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

fn get_free_ip(db: &Leased4Table, subnet: &Dhcp4SubnetConfig, mac_address: MacAddress) -> Result<Ipv4Addr> {
    let range = Ipv4AddrRange::new(subnet.range.0, subnet.range.1);
    for target in range {
        let Ok(false) = db.is_exist(&target) else {
            continue;
        };
        if let Ok(ip_addr) = db.lease(
            target,
            mac_address,
            Local::now().add(Duration::seconds(subnet.address_lease_time as i64)),
        ) {
            return Ok(ip_addr);
        }
    }
    bail!("No free ip")
}

fn get_new_ip(subnet: &Dhcp4SubnetConfig, mac_address: MacAddress) -> Result<Ipv4Addr> {
    let db = Db::open().leased()?;
    if let Some(ip_addr) = db.search_mac_address(mac_address) {
        return Ok(ip_addr);
    }
    let ip = get_free_ip(&db, subnet, mac_address)?;
    Ok(ip)
}
