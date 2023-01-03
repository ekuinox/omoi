mod conf;
mod db;
mod dhcp;
mod http;

use self::conf::OMOI_CONFIG;
use anyhow::{anyhow, ensure, Result};

#[tokio::main]
async fn main() -> Result<()> {
    ensure!(OMOI_CONFIG.dhcp4.subnets.len() == 1);
    let r = tokio::select! {
        r = dhcp::v4::serve() => {r},
        r = http::serve() => {r},
        r = tokio::signal::ctrl_c() => {r.map_err(|e| anyhow!(e))},
    };
    if let Err(e) = r {
        eprintln!("{e}");
    }
    Ok(())
}
