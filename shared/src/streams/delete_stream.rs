use crate::bytes_serializable::BytesSerializable;
use crate::command::DELETE_STREAM;
use crate::error::Error;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub struct DeleteStream {
    pub stream_id: u32,
}

impl FromStr for DeleteStream {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parts = input.split('|').collect::<Vec<&str>>();
        if parts.len() != 1 {
            return Err(Error::InvalidCommand);
        }

        let stream_id = parts[0].parse::<u32>()?;
        if stream_id == 0 {
            return Err(Error::InvalidStreamId);
        }

        Ok(DeleteStream { stream_id })
    }
}

impl BytesSerializable for DeleteStream {
    type Type = DeleteStream;

    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4);
        bytes.extend(self.stream_id.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::Type, Error> {
        if bytes.len() != 4 {
            return Err(Error::InvalidCommand);
        }

        let stream_id = u32::from_le_bytes(bytes.try_into()?);
        if stream_id == 0 {
            return Err(Error::InvalidStreamId);
        }

        Ok(DeleteStream { stream_id })
    }
}

impl Display for DeleteStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} → stream ID: {}", DELETE_STREAM, self.stream_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let is_empty = false;
        let command = DeleteStream { stream_id: 1 };

        let bytes = command.as_bytes();
        let stream_id = u32::from_le_bytes(bytes[..4].try_into().unwrap());

        assert_eq!(bytes.is_empty(), is_empty);
        assert_eq!(stream_id, command.stream_id);
    }

    #[test]
    fn should_be_deserialized_from_bytes() {
        let is_ok = true;
        let stream_id = 1u32;
        let bytes = stream_id.to_le_bytes();
        let command = DeleteStream::from_bytes(&bytes);
        assert_eq!(command.is_ok(), is_ok);

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
    }

    #[test]
    fn should_be_read_from_string() {
        let is_ok = true;
        let stream_id = 1u32;
        let input = format!("{}", stream_id);
        let command = DeleteStream::from_str(&input);
        assert_eq!(command.is_ok(), is_ok);

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
    }
}