use crate::conf::OMOI_CONFIG;
use anyhow::Result;
use chrono::{DateTime, Local};
use mac_address::MacAddress;
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, ops::Deref, path::Path};

#[derive(Clone, Debug)]
pub struct Db {
    inner: sled::Db,
}

impl Db {
    pub fn try_open(path: &Path) -> Result<Db> {
        let inner = sled::open(path)?;
        Ok(Db { inner })
    }
    pub fn open() -> Db {
        Self::try_open(&OMOI_CONFIG.common.database_dir).expect("open db error")
    }
    pub fn leased(&self) -> Result<Leased4Table> {
        let tree = self.open_tree("LEASED4")?;
        Ok(Leased4Table { inner: tree })
    }
}

impl Deref for Db {
    type Target = sled::Db;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct Leased4Table {
    inner: sled::Tree,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub struct Leased4Detail {
    ttl: DateTime<Local>,
    mac_address: MacAddress,
    ip_addr: Ipv4Addr,
}

impl Leased4Table {
    pub fn search_mac_address(&self, mac_address: MacAddress) -> Option<Ipv4Addr> {
        let detail = self
            .inner
            .into_iter()
            .flatten()
            .flat_map(|(_, v)| toml::from_slice::<Leased4Detail>(&v))
            .find(|detail| detail.mac_address == mac_address);
        detail.map(|detail| detail.ip_addr)
    }

    pub fn is_exist(&self, ip_addr: &Ipv4Addr) -> Result<bool> {
        let Some(v) = self.inner.get(ip_addr.octets())? else {
            return Ok(false);
        };
        let Ok(detail) = toml::from_slice::<Leased4Detail>(&v) else {
            return Ok(false);
        };
        Ok(Local::now() < detail.ttl)
    }
    pub fn lease(
        &self,
        ip_addr: Ipv4Addr,
        mac_address: MacAddress,
        ttl: DateTime<Local>,
    ) -> Result<Ipv4Addr> {
        let detail = Leased4Detail {
            mac_address,
            ttl,
            ip_addr,
        };
        let text = toml::to_string(&detail)?;
        let _ = self.inner.insert(ip_addr.octets(), text.as_bytes())?;
        Ok(ip_addr)
    }
}
