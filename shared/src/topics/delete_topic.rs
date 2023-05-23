use crate::bytes_serializable::BytesSerializable;
use crate::command::DELETE_TOPIC;
use crate::error::Error;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub struct DeleteTopic {
    pub stream_id: u32,
    pub topic_id: u32,
}

impl FromStr for DeleteTopic {
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

        let topic_id = parts[1].parse::<u32>()?;
        if topic_id == 0 {
            return Err(Error::InvalidTopicId);
        }

        Ok(DeleteTopic {
            stream_id,
            topic_id,
        })
    }
}

impl BytesSerializable for DeleteTopic {
    type Type = DeleteTopic;

    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8);
        bytes.extend(self.stream_id.to_le_bytes());
        bytes.extend(self.topic_id.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self::Type, Error> {
        if bytes.len() != 8 {
            return Err(Error::InvalidCommand);
        }

        let stream_id = u32::from_le_bytes(bytes[..4].try_into()?);
        if stream_id == 0 {
            return Err(Error::InvalidStreamId);
        }

        let topic_id = u32::from_le_bytes(bytes[4..8].try_into()?);
        if topic_id == 0 {
            return Err(Error::InvalidTopicId);
        }

        Ok(DeleteTopic {
            stream_id,
            topic_id,
        })
    }
}

impl Display for DeleteTopic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} → stream ID: {}, topic ID: {}",
            DELETE_TOPIC, self.stream_id, self.topic_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let is_empty = false;
        let command = DeleteTopic {
            stream_id: 1,
            topic_id: 2,
        };

        let bytes = command.as_bytes();
        let stream_id = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        let topic_id = u32::from_le_bytes(bytes[4..8].try_into().unwrap());

        assert_eq!(bytes.is_empty(), is_empty);
        assert_eq!(stream_id, command.stream_id);
        assert_eq!(topic_id, command.topic_id);
    }

    #[test]
    fn should_be_deserialized_from_bytes() {
        let is_ok = true;
        let stream_id = 1u32;
        let topic_id = 2u32;
        let bytes = [stream_id.to_le_bytes(), topic_id.to_le_bytes()].concat();
        let command = DeleteTopic::from_bytes(&bytes);
        assert_eq!(command.is_ok(), is_ok);

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.topic_id, topic_id);
    }

    #[test]
    fn should_be_read_from_string() {
        let is_ok = true;
        let stream_id = 1u32;
        let topic_id = 2u32;
        let input = format!("{}|{}", stream_id, topic_id);
        let command = DeleteTopic::from_str(&input);
        assert_eq!(command.is_ok(), is_ok);

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.topic_id, topic_id);
    }
}