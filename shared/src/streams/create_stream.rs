use crate::bytes_serializable::BytesSerializable;
use crate::command::CREATE_STREAM;
use crate::error::Error;
use std::fmt::Display;
use std::str::{from_utf8, FromStr};

pub const MAX_NAME_LENGTH: usize = 100;

#[derive(Debug)]
pub struct CreateStream {
    pub stream_id: u32,
    pub name: String,
}

impl FromStr for CreateStream {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parts = input.split('|').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(Error::InvalidCommand);
        }

        let stream_id = parts[0].parse::<u32>()?;
        if stream_id == 0 {
            return Err(Error::InvalidStreamId);
        }

        let name = parts[1].to_string();
        if name.len() > MAX_NAME_LENGTH {
            return Err(Error::InvalidStreamName);
        }

        Ok(CreateStream { stream_id, name })
    }
}

impl BytesSerializable for CreateStream {
    type Type = CreateStream;

    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.name.len());
        bytes.extend_from_slice(&self.stream_id.to_le_bytes());
        bytes.extend_from_slice(self.name.as_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::Type, Error> {
        if bytes.len() < 5 {
            return Err(Error::InvalidCommand);
        }

        let stream_id = u32::from_le_bytes(bytes[..4].try_into()?);
        if stream_id == 0 {
            return Err(Error::InvalidStreamId);
        }

        let name = from_utf8(&bytes[4..])?.to_string();
        if name.len() > MAX_NAME_LENGTH {
            return Err(Error::InvalidStreamName);
        }

        Ok(CreateStream { stream_id, name })
    }
}

impl Display for CreateStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} → stream ID: {}, name: {}",
            CREATE_STREAM, self.stream_id, self.name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let is_empty = false;
        let command = CreateStream {
            stream_id: 1,
            name: "test".to_string(),
        };

        let bytes = command.as_bytes();
        let stream_id = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        let name = from_utf8(&bytes[4..]).unwrap();

        assert_eq!(bytes.is_empty(), is_empty);
        assert_eq!(stream_id, command.stream_id);
        assert_eq!(name, command.name);
    }

    #[test]
    fn should_be_deserialized_from_bytes() {
        let is_ok = true;
        let stream_id = 1u32;
        let name = "test".to_string();
        let bytes = [&stream_id.to_le_bytes(), name.as_bytes()].concat();
        let command = CreateStream::from_bytes(&bytes);
        assert_eq!(command.is_ok(), is_ok);

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.name, name);
    }

    #[test]
    fn should_be_read_from_string() {
        let is_ok = true;
        let stream_id = 1u32;
        let name = "test".to_string();
        let input = format!("{}|{}", stream_id, name);
        let command = CreateStream::from_str(&input);
        assert_eq!(command.is_ok(), is_ok);

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.name, name);
    }
}