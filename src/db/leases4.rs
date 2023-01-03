use std::net::Ipv4Addr;

use anyhow::{bail, Result};
use chrono::{DateTime, Local};
use ipnet::Ipv4AddrRange;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct Leases4Record {
    pub hardware_address: Vec<u8>,
    pub ip_addr: Ipv4Addr,
    pub ttl: DateTime<Local>,
}

impl Leases4Record {
    pub fn is_expired(&self) -> bool {
        Local::now() >= self.ttl
    }
}

#[derive(Clone, Debug)]
pub struct Leases4Tree {
    inner: sled::Tree,
}

impl Leases4Tree {
    pub fn new(inner: sled::Tree) -> Leases4Tree {
        Leases4Tree { inner }
    }

    pub fn generate_key(address: &Ipv4Addr) -> Vec<u8> {
        address.octets().to_vec()
    }

    pub fn get_by_ip(&self, address: &Ipv4Addr) -> Result<Leases4Record> {
        let key = Self::generate_key(address);
        let Some(value) = self.inner.get(key)? else {
            bail!("Empty key {address}");
        };
        let record: Leases4Record = bincode::deserialize(&value)?;
        Ok(record)
    }

    pub fn get_by_hw(&self, address: &[u8]) -> Result<Leases4Record> {
        for (key, value) in self.inner.into_iter().flatten() {
            let Ok(record) = bincode::deserialize::<Leases4Record>(&value) else {
                eprintln!("Key={key:?} deserialize error");
                continue;
            };
            if record.hardware_address == address {
                return Ok(record);
            }
        }
        bail!("Not found");
    }

    pub fn all(&self) -> Vec<Leases4Record> {
        self.inner
            .into_iter()
            .flatten()
            .flat_map(|(_, value)| bincode::deserialize(&value))
            .collect()
    }

    pub fn suggest(&self, hw_address: &[u8], start: Ipv4Addr, end: Ipv4Addr) -> Result<Ipv4Addr> {
        if let Ok(record) = self.get_by_hw(hw_address) {
            return Ok(record.ip_addr);
        }

        for addr in Ipv4AddrRange::new(start, end) {
            if self
                .get_by_ip(&addr)
                .map(|record| record.is_expired())
                .unwrap_or(true)
            // 壊れているレコードは空きとみなす
            {
                return Ok(addr);
            }
        }
        bail!("No empty address");
    }

    pub fn acquire(
        &self,
        hw_addr: Vec<u8>,
        ip_addr: Ipv4Addr,
        ttl: DateTime<Local>,
    ) -> Result<Leases4Record> {
        let key = Self::generate_key(&ip_addr);
        let record = Leases4Record {
            hardware_address: hw_addr,
            ip_addr,
            ttl,
        };
        let serialized = bincode::serialize(&record)?;
        let _ = self.inner.insert(key, serialized)?;
        Ok(record)
    }
}

#[test]
fn encode_decode_test() {
    let now = Local::now();
    let record = Leases4Record {
        hardware_address: vec![1, 2, 3],
        ip_addr: "192.168.1.1".parse().unwrap(),
        ttl: now,
    };
    let serialized = bincode::serialize(&record).unwrap();
    let deserialized = bincode::deserialize(&serialized).unwrap();
    assert_eq!(record, deserialized);
}
