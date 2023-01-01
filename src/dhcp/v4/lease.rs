use std::net::Ipv4Addr;

use crate::conf::OMOI_CONFIG;
use anyhow::{bail, Result};
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
        let host = match host {
            Some(host) => host,
            None => {
                // TODO: 適当に割り当てる
                bail!("Unimplemented");
            }
        };
        let resp = LeasedResponse {
            mac_address: self.mac_address,
            ip_addr: host.fixed_address,
            broadcast_address: subnet.broadcast_address,
            subnet_mask: subnet.netmask,
            domain_name_servers: subnet.domain_name_servers.clone(),
            address_lease_time: subnet.address_lease_time,
            routers: subnet.routers.clone(),
        };
        Ok(resp)
    }
}
