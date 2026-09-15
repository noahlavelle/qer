use redisclient::aio::MultiplexedConnection;

use crate::engine::StoreError;

mod metadata;
mod scripts;

pub use metadata::RedisMetadataStore;

pub async fn connect(redis_url: &str) -> Result<MultiplexedConnection, StoreError> {
    let r = redisclient::Client::open(redis_url)?;
    let client = r.get_multiplexed_async_connection().await?;

    Ok(client)
}
