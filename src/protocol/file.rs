use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Deserializer, Serialize};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DhcpMessageFileField([u8; 128]);

struct DhcpMessageFileFieldVisitor;

impl<'de> Visitor<'de> for DhcpMessageFileFieldVisitor {
    type Value = DhcpMessageFileField;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("invalid")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut sname = DhcpMessageFileField([0u8; 128]);
        for i in 0..128 {
            if let Some(elm) = seq.next_element::<u8>()? {
                sname.0[i] = elm;
            } else {
                return Err(serde::de::Error::custom("invalid file"));
            }
        }
        Ok(sname)
    }
}

impl<'de> Deserialize<'de> for DhcpMessageFileField {
    fn deserialize<D>(deserializer: D) -> Result<DhcpMessageFileField, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(128, DhcpMessageFileFieldVisitor)
    }
}

impl Serialize for DhcpMessageFileField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut t = serializer.serialize_tuple(128)?;
        for n in self.0 {
            t.serialize_element(&n)?;
        }
        t.end()
    }
}
