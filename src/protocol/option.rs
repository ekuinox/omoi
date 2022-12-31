use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Deserializer, Serialize};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DhcpOption {
    Pad,                    // 0
    SubnetMask([u8; 4]),    // 1
    TimeOffset([u8; 4]),    // 2
    Router(Vec<u8>),        // 3
    TimeServer(Vec<u8>),    // 4
    NameServer(Vec<u8>),    // 5
    DomainServer(Vec<u8>),  // 6
    LogServer(Vec<u8>),     // 7
    QuotesServer(Vec<u8>),  // 8
    LrpServer(Vec<u8>),     // 9
    ImpressServer(Vec<u8>), // 10
    RlpServer(Vec<u8>),     // 11
    Hostname(Vec<u8>),      // 12
    BootFileSize(u16),      // 13
    MeritDumpFile(Vec<u8>), // 14
    DomainName(Vec<u8>),    // 15
    SwapServer(Vec<u8>),    // 16
    RootPath(Vec<u8>),      // 17
    ExtensionFile(Vec<u8>), // 18
    ForwardOnOff(u8),       // 19
    SrcRteOnOff(u8),        // 20
    PolicyFilter(Vec<u8>),  // 21
    MaxDgAssembly(u16),     // 22
    DefaultIpTtl(u8),       // 23
    MtuTimeout(u32),        // 24
    MtuPlateu(Vec<u8>),     // 25
    MtuInterface(u16),      // 26

    // skip
    DhcpMessageType(u8), // 53
    DhcpServerId(u32),   // 54
    DhcpParameterList(Vec<u8>), // 55

    // skip
    RemovedOrUndefined(u8, u8, Vec<u8>), // 99 or 102 ~ 107 or ...

    // skip
    End, // 255
}

mod tag {
    pub const PAD: u8 = 0;
    pub const DHCP_MESSAGE_TYPE: u8 = 53;
    pub const DHCP_SERVER_ID: u8 = 54;
    pub const PARAMETER_LIST: u8 = 55;
    pub const END: u8 = 255;
}
pub const MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];

struct DhcpOptionVisitor;

impl<'de> Visitor<'de> for DhcpOptionVisitor {
    type Value = DhcpOption;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("invalid")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let tag = seq.next_element::<u8>()?.unwrap();
        match tag {
            tag::PAD => Ok(DhcpOption::Pad),
            tag::DHCP_MESSAGE_TYPE => {
                let size = seq.next_element::<u8>()?;
                if size != Some(std::mem::size_of::<u8>() as u8) {
                    return Err(serde::de::Error::missing_field("DHCP_MESSAGE_TYPE FIELD size"))
                }
                if let Some(ty) = seq.next_element()? {
                    Ok(DhcpOption::DhcpMessageType(ty))
                } else {
                    Err(serde::de::Error::missing_field("DHCP_MESSAGE_TYPE FIELD value"))
                }
            }
            tag::DHCP_SERVER_ID => {
                let size = seq.next_element::<u8>()?;
                if size != Some(std::mem::size_of::<u8>() as u8) {
                    return Err(serde::de::Error::missing_field("DHCP_SERVER_ID FIELD size"))
                }
                if let Some(id) = seq.next_element()? {
                    Ok(DhcpOption::DhcpServerId(id))
                } else {
                    Err(serde::de::Error::missing_field("DHCP_MESSAGE_TYPE FIELD value"))
                }
            }
            tag::PARAMETER_LIST => {
                let Some(size) = seq.next_element::<u8>()? else {
                    return Err(serde::de::Error::missing_field("DHCP_SERVER_ID FIELD size"))
                };
                let mut list = Vec::with_capacity(size as usize);
                for _ in 0..size as usize {
                    if let Some(param) = seq.next_element()? {
                        list.push(param);
                    } else {
                        return Err(serde::de::Error::missing_field("PARAMETER_LIST FIELD value"));
                    }
                }
                Ok(DhcpOption::DhcpParameterList(list))
            }
            tag::END => Ok(DhcpOption::End),
            _ => {
                let size = seq.next_element::<u8>()?.unwrap();
                let mut buffer = vec![0u8; size as usize];
                for i in 0..size as usize {
                    if let Some(elm) = seq.next_element()? {
                        buffer[i] = elm;
                    }
                }
                Ok(DhcpOption::RemovedOrUndefined(tag, size, buffer))
            }
        }
    }
}

impl<'de> Deserialize<'de> for DhcpOption {
    fn deserialize<D>(deserializer: D) -> Result<DhcpOption, D::Error>
    where
        D: Deserializer<'de>,
    {
        // これnext_elementしたときにlen以上だと取れないんですね...
        // deserializer.deserialize_bytes(DhcpOptionVisitor)
        deserializer.deserialize_tuple(usize::MAX, DhcpOptionVisitor)
    }
}

impl Serialize for DhcpOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut t = serializer.serialize_tuple(3)?;
        match self {
            Self::DhcpMessageType(ty) => {
                t.serialize_element(&tag::DHCP_MESSAGE_TYPE)?;
                t.serialize_element(&(std::mem::size_of::<u8>() as u8))?;
                t.serialize_element(ty)?;
            }
            _ => unimplemented!(),
        }
        t.end()
    }
}

#[derive(Clone, Debug)]
pub struct DhcpOptions(Vec<DhcpOption>);

struct DhcpOptionsVisitor;

impl<'de> Visitor<'de> for DhcpOptionsVisitor {
    type Value = DhcpOptions;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("invalid")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let magic_cookie = seq.next_element::<[u8; 4]>()?;
        if !magic_cookie.map(|c| c == MAGIC_COOKIE).unwrap_or(false) {
            return Err(serde::de::Error::missing_field("MAGIC_COOKIE"));
        }
        let mut options = vec![];
        while let Some(option) = seq.next_element::<DhcpOption>()? {
            options.push(option.clone());
            if DhcpOption::End == option {
                break;
            }
        }
        Ok(DhcpOptions(options))
    }
}

impl<'de> Deserialize<'de> for DhcpOptions {
    fn deserialize<D>(deserializer: D) -> Result<DhcpOptions, D::Error>
    where
        D: Deserializer<'de>,
    {
        // seqにしたいんですけど、seqにしてしまうと先頭がサイズとして取られちゃう！！
        deserializer.deserialize_tuple(usize::MAX, DhcpOptionsVisitor)
    }
}

impl Serialize for DhcpOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut t = serializer.serialize_tuple(self.0.len())?;
        for option in &self.0 {
            t.serialize_element(option)?;
        }
        t.end()
    }
}
