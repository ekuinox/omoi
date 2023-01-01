use anyhow::Result;
use mac_address::MacAddress;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufReader, Read},
    net::Ipv4Addr,
    path::PathBuf,
};

const DEFAULT_OMOI_CONFIG_PATH: &str = "/etc/omoi.conf";
const OMOI_CONFIG_PATH_ENV_KEY: &str = "OMOI_CONFIG_PATH";
pub static OMOI_CONFIG: Lazy<OmoiConfig> = Lazy::new(|| OmoiConfig::load());

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct CommonConfig {
    pub database_dir: PathBuf,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Dhcp4SubnetConfig {
    pub subnet: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub range: (Ipv4Addr, Ipv4Addr),
    pub domain_name_servers: Vec<Ipv4Addr>,
    pub routers: Vec<Ipv4Addr>,
    pub broadcast_address: Ipv4Addr,
    pub max_lease_time: u64,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Dhcp4HostConfig {
    pub name: String,
    pub hardware_ethernet: MacAddress,
    pub fixed_address: Ipv4Addr,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Dhcp4Config {
    #[serde(rename = "subnet")]
    pub subnets: Vec<Dhcp4SubnetConfig>,
    #[serde(rename = "host")]
    pub hosts: Vec<Dhcp4HostConfig>,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct OmoiConfig {
    pub common: CommonConfig,
    pub dhcp4: Dhcp4Config,
}

impl OmoiConfig {
    fn try_load() -> Result<OmoiConfig> {
        let file = match std::env::var(OMOI_CONFIG_PATH_ENV_KEY) {
            Ok(path) => File::open(path)?,
            Err(_) => File::open(DEFAULT_OMOI_CONFIG_PATH)?,
        };
        let size = file
            .metadata()
            .map(|meta| meta.len() as usize)
            .unwrap_or(1024);
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::with_capacity(size);
        let size = reader.read_to_end(&mut buffer)?;
        let config: OmoiConfig = toml::from_slice(&buffer[..size])?;
        Ok(config)
    }
    fn load() -> OmoiConfig {
        let config = Self::try_load().expect("OmoiConfig Read Error");
        config
    }
}

#[test]
fn parse_test() {
    use std::path::Path;
    const TOML_TEXT: &str = r#"
[common]
database-dir = "omoi-db"

[dhcp4]
domain-name = "example.local"

[[dhcp4.subnet]]
subnet = "192.168.0.1"
netmask = "255.255.255.0"
range = ["192.168.0.101", "192.168.0.250"]
domain-name-servers = ["192.168.0.1"]
routers = ["192.168.0.1"]
broadcast-address = "192.168.0.255"
max-lease-time = 172800

[[dhcp4.host]]
name = "host1"
hardware-ethernet = "00:00:00:11:11:11"
fixed-address = "192.168.0.11"
    "#;
    let expected = OmoiConfig {
        common: CommonConfig {
            database_dir: Path::new("omoi-db").to_owned(),
        },
        dhcp4: Dhcp4Config {
            subnets: vec![Dhcp4SubnetConfig {
                subnet: Ipv4Addr::new(192, 168, 0, 1),
                netmask: Ipv4Addr::new(255, 255, 255, 0),
                range: (
                    Ipv4Addr::new(192, 168, 0, 101),
                    Ipv4Addr::new(192, 168, 0, 250),
                ),
                domain_name_servers: vec![Ipv4Addr::new(192, 168, 0, 1)],
                routers: vec![Ipv4Addr::new(192, 168, 0, 1)],
                broadcast_address: Ipv4Addr::new(192, 168, 0, 255),
                max_lease_time: 172800,
            }],
            hosts: vec![Dhcp4HostConfig {
                name: "host1".to_string(),
                hardware_ethernet: MacAddress::new([0x00, 0x00, 0x00, 0x11, 0x11, 0x11]),
                fixed_address: Ipv4Addr::new(192, 168, 0, 11),
            }],
        },
    };

    let config = toml::from_str::<OmoiConfig>(TOML_TEXT);
    assert_eq!(Ok(expected), config);
}
