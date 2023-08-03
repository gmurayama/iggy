use crate::binary;
use crate::client::ConsumerOffsetClient;
use crate::consumer_offsets::get_consumer_offset::GetConsumerOffset;
use crate::consumer_offsets::store_consumer_offset::StoreConsumerOffset;
use crate::error::Error;
use crate::models::offset::Offset;
use crate::quic::client::QuicClient;
use async_trait::async_trait;

#[async_trait]
impl ConsumerOffsetClient for QuicClient {
    async fn store_consumer_offset(&self, command: &StoreConsumerOffset) -> Result<(), Error> {
        binary::consumer_offsets::store_consumer_offset(self, command).await
    }

    async fn get_consumer_offset(&self, command: &GetConsumerOffset) -> Result<Offset, Error> {
        binary::consumer_offsets::get_consumer_offset(self, command).await
    }
}