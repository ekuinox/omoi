mod leases4;

use anyhow::Result;
use std::{ops::Deref, path::Path};

use self::leases4::Leases4Tree;
use crate::conf::OMOI_CONFIG;

pub use self::leases4::Leases4Record;

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
    pub fn leases_tree(&self) -> Result<Leases4Tree> {
        let tree = self.open_tree("LEASES4")?;
        Ok(Leases4Tree::new(tree))
    }
}

impl Deref for Db {
    type Target = sled::Db;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
