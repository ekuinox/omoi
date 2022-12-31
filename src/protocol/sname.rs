use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Deserializer, Serialize};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DhcpMessageSnameField([u8; 64]);

struct DhcpMessageSnameFieldVisitor;

impl<'de> Visitor<'de> for DhcpMessageSnameFieldVisitor {
    type Value = DhcpMessageSnameField;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("invalid")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut sname = DhcpMessageSnameField([0u8; 64]);
        for i in 0..64 {
            if let Some(elm) = seq.next_element::<u8>()? {
                sname.0[i] = elm;
            } else {
                return Err(serde::de::Error::custom("invalid sname"));
            }
        }
        Ok(sname)
    }
}

impl<'de> Deserialize<'de> for DhcpMessageSnameField {
    fn deserialize<D>(deserializer: D) -> Result<DhcpMessageSnameField, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(64, DhcpMessageSnameFieldVisitor)
    }
}

impl Serialize for DhcpMessageSnameField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut t = serializer.serialize_tuple(64)?;
        for n in self.0 {
            t.serialize_element(&n)?;
        }
        t.end()
    }
}
