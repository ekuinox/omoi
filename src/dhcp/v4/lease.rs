use crate::{
    conf::{Dhcp4SubnetConfig, OMOI_CONFIG},
    db::Db,
};
use anyhow::{bail, Result};
use chrono::{Duration, Local};
use once_cell::sync::Lazy;
use std::{collections::HashMap, net::Ipv4Addr, ops::Add, sync::Mutex};

static OFFERED_ADDRESSES: Lazy<Mutex<HashMap<u32, Ipv4Addr>>> =
    Lazy::new(|| Mutex::new(HashMap::<u32, Ipv4Addr>::new()));

pub struct LeaseRequest {
    xid: u32,
    hardware_address: Vec<u8>,
}

pub struct LeasedResponse {
    pub hardware_address: Vec<u8>,
    pub ip_addr: Ipv4Addr,
    pub broadcast_address: Ipv4Addr,
    pub subnet_mask: Ipv4Addr,
    pub routers: Vec<Ipv4Addr>,
    pub domain_name_servers: Vec<Ipv4Addr>,
    pub address_lease_time: u32,
}

impl LeaseRequest {
    pub fn new(xid: u32, hardware_address: Vec<u8>) -> LeaseRequest {
        LeaseRequest {
            xid,
            hardware_address,
        }
    }

    pub fn offer(self) -> Result<LeasedResponse> {
        let Some(subnet) = OMOI_CONFIG.dhcp4.subnets.first() else {
            bail!("subnets is empty");
        };
        let host = OMOI_CONFIG
            .dhcp4
            .hosts
            .iter()
            .find(|host| host.hardware_ethernet.bytes().to_vec() == self.hardware_address);
        let ip_addr = match host {
            Some(host) => host.fixed_address,
            None => offer(self.xid, &subnet, &self.hardware_address)?,
        };
        let resp = LeasedResponse {
            hardware_address: self.hardware_address,
            ip_addr,
            broadcast_address: subnet.broadcast_address,
            subnet_mask: subnet.netmask,
            domain_name_servers: subnet.domain_name_servers.clone(),
            address_lease_time: subnet.address_lease_time,
            routers: subnet.routers.clone(),
        };
        Ok(resp)
    }

    pub fn ack(self) -> Result<LeasedResponse> {
        let Some(subnet) = OMOI_CONFIG.dhcp4.subnets.first() else {
            bail!("subnets is empty");
        };
        let host = OMOI_CONFIG
            .dhcp4
            .hosts
            .iter()
            .find(|host| host.hardware_ethernet.bytes().to_vec() == self.hardware_address);
        let ip_addr = match host {
            Some(host) => host.fixed_address,
            None => ack(self.xid, &subnet, &self.hardware_address)?,
        };
        let resp = LeasedResponse {
            hardware_address: self.hardware_address,
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

fn offer(xid: u32, subnet: &Dhcp4SubnetConfig, hardware_address: &[u8]) -> Result<Ipv4Addr> {
    let db = Db::open().leases_tree()?;
    let ip = db.suggest(hardware_address, subnet.range.0, subnet.range.1)?;
    let Ok(mut offered) = OFFERED_ADDRESSES.lock() else {
        bail!("OFFERED_ADDRESSES lock failed");
    };
    offered.insert(xid, ip);
    Ok(ip)
}

fn ack(xid: u32, subnet: &Dhcp4SubnetConfig, hardware_address: &[u8]) -> Result<Ipv4Addr> {
    let db = Db::open().leases_tree()?;
    let acquire = |addr: Ipv4Addr| -> Result<Ipv4Addr> {
        let record = db.acquire(
            hardware_address.to_vec(),
            addr,
            Local::now().add(Duration::seconds(subnet.address_lease_time.into())),
        )?;
        Ok(record.ip_addr)
    };
    match OFFERED_ADDRESSES
        .lock()
        .map(|mut offered| offered.remove(&xid))
    {
        Ok(Some(addr)) => acquire(addr),
        _ => {
            let addr = db.suggest(hardware_address, subnet.range.0, subnet.range.1)?;
            acquire(addr)
        }
    }
}
