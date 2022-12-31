mod dhcp;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tokio::select! {
        _ = dhcp::v4::serve() => {},
        _ = tokio::signal::ctrl_c() => {},
    }
    Ok(())
}
