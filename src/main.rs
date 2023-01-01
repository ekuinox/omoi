mod conf;
mod dhcp;

use self::conf::OMOI_CONFIG;
use anyhow::{Result, ensure};

#[tokio::main]
async fn main() -> Result<()> {
    ensure!(OMOI_CONFIG.dhcp4.subnets.len() == 1);
    dbg!(&OMOI_CONFIG);
    tokio::select! {
        _ = dhcp::v4::serve() => {},
        _ = tokio::signal::ctrl_c() => {},
    }
    Ok(())
}
