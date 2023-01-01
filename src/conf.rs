use mac_address::MacAddress;
use serde::Deserialize;
use std::{net::Ipv4Addr, path::PathBuf};

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct OmoiConfig {
    database_dir: PathBuf,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Dhcp4SubnetConfig {
    subnet: Ipv4Addr,
    netmask: Ipv4Addr,
    range: (Ipv4Addr, Ipv4Addr),
    domain_name_servers: Vec<Ipv4Addr>,
    routers: Vec<Ipv4Addr>,
    broadcast_address: Ipv4Addr,
    max_lease_time: u64,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Dhcp4HostConfig {
    name: String,
    hardware_ethernet: MacAddress,
    fixed_address: Ipv4Addr,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Dhcp4Config {
    #[serde(rename = "subnet")]
    subnets: Vec<Dhcp4SubnetConfig>,
    #[serde(rename = "host")]
    hosts: Vec<Dhcp4HostConfig>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    omoi: OmoiConfig,
    dhcp4: Dhcp4Config,
}

#[test]
fn parse_test() {
    const TOML_TEXT: &str = r#"
[omoi]
database-dir = "omoi-db"

[dhcp4]
domain-name = "kuaga.local"

[[dhcp4.subnet]]
subnet = "192.168.1.0"
netmask = "255.255.255.0"
range = ["192.168.1.201", "192.168.1.240"]
domain-name-servers = ["192.168.1.15"]
routers = ["192.168.1.1"]
broadcast-address = "192.168.1.255"
max-lease-time = 172800

[[dhcp4.host]]
name = "aoi"
hardware-ethernet = "dc:a6:32:e6:0f:44"
fixed-address = "192.168.1.15"
    "#;
    let config = toml::from_str::<Config>(TOML_TEXT).unwrap();
    dbg!(config);
}
