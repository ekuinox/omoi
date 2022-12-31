mod file;
mod option;
mod sname;

use self::{file::DhcpMessageFileField, option::DhcpOptions, sname::DhcpMessageSnameField};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DhcpMessageFormat {
    pub op: u8,
    pub htype: u8,
    pub hlen: u8,
    pub hops: u8,
    pub xid: u32,
    pub secs: u16,
    pub flags: u16,
    pub ciaddr: u32,
    pub yiaddr: u32,
    pub siaddr: u32,
    pub giaddr: u32,
    pub chaddr: [u8; 16],
    pub sname: DhcpMessageSnameField,
    pub file: DhcpMessageFileField,
    pub options: DhcpOptions,
}

#[test]
fn test_message() {
    use bincode::Options;
    const BUFFER: [u8; 251] = [
        1, // op
        1, // htype
        6, // hlen
        0, // hops
        2, 44, 180, 37, // xid
        0, 0, // secs
        0, 0, // flags
        0, 0, 0, 0, // ciaddr
        0, 0, 0, 0, // yiaddr
        0, 0, 0, 0, // siaddr
        0, 0, 0, 0, // giaddr
        0, 0, 0, 17, 17, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // chaddr
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, // sname
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, // file
        99, 130, 83, 99, // magic cookie
        53, 1, 1, // option dhcp message type
        55, 5, 1, 28, 3, 15, 6,   // parameter list
        255, // options
    ];
    let a = bincode::options()
        .with_big_endian()
        .allow_trailing_bytes()
        .with_fixint_encoding()
        .deserialize::<DhcpMessageFormat>(&BUFFER)
        .unwrap();
    dbg!(a);
}
