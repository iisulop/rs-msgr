use prost::{DecodeError, EncodeError, Message as ProstMessage};
use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum MsgrError {
    #[error("Cannot read from socket")]
    SocketReadError(#[from] std::io::Error),
    #[error("Cannot serialize message")]
    MessageSerializationError(#[from] prost::EncodeError),
    #[error("Cannot deserialize message")]
    MessageDeserializationError(#[from] prost::DecodeError),
}

include!(concat!(env!("OUT_DIR"), "/msgr.types.message.rs"));

pub fn serialize_message(contents: String) -> Result<Vec<u8>, EncodeError> {
    let msg = Message {
        recipient: 0,
        sender: 0,
        content: Some(MessageContent { contents }),
    };
    let mut buf: Vec<u8> = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf)?;
    Ok(buf)
}

pub fn deserialize_message(buf: &[u8]) -> Result<Message, DecodeError> {
    let msg = Message::decode(buf)?;
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use crate::{deserialize_message, serialize_message};

    #[test]
    fn ser_de() {
        let contents = String::from("A test message");
        let serialized = serialize_message(contents.clone()).unwrap();
        println!("{:#?}", serialized);
        let deserialized = deserialize_message(&serialized).unwrap();
        assert_eq!(contents, deserialized.content.unwrap().contents);
    }
}
