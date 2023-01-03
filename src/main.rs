mod conf;
mod db;
mod dhcp;
mod http;

use self::conf::OMOI_CONFIG;
use anyhow::{ensure, Result};

#[tokio::main]
async fn main() -> Result<()> {
    ensure!(OMOI_CONFIG.dhcp4.subnets.len() == 1);
    tokio::select! {
        _ = dhcp::v4::serve() => {},
        _ = http::serve() => {},
        _ = tokio::signal::ctrl_c() => {},
    }
    Ok(())
}
