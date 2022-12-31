use anyhow::Result;
use dhcproto::{Decodable, v4::Message, Decoder};

pub fn decode(buffer: &[u8]) -> Result<Message> {
    let mut decoder = Decoder::new(buffer);
    let message = Message::decode(&mut decoder)?;
    Ok(message)
}
