use crate::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use crate::settings::structs::RedisConfig;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

pub struct RedisClient {
  connection: MultiplexedConnection,
}

impl RedisClient {
  /// Initializes Redis client and connects immediately
  pub async fn new(config: RedisConfig) -> FastJobResult<Self> {
    let client = redis::Client::open(config.connection.clone())
      .with_fastjob_type(FastJobErrorType::RedisConnectionFailed)?;

    let mut conn = client
      .get_multiplexed_async_connection()
      .await
      .with_fastjob_type(FastJobErrorType::RedisConnectionFailed)?;
    let _: String = redis::cmd("PING")
        .query_async(&mut conn)
        .await
        .with_fastjob_type(FastJobErrorType::RedisConnectionFailed)?;

    println!("Connected to Redis at {}", config.connection);

    Ok(Self {
      connection: conn,
    })
  }

  /// Set a JSON-serialized value with expiration (unit second)
  pub async fn set_value_with_expiry<T: serde::Serialize>(
    &mut self,
    key: &str,
    value: T,
    expiry: usize,
  ) -> FastJobResult<()> {
    let value_str =
      serde_json::to_string(&value).with_fastjob_type(FastJobErrorType::SerializationFailed)?;

    let result: redis::RedisResult<()> = self.connection.set_ex(key, value_str, expiry as u64).await;

    result.map_err(|_| FastJobErrorType::RedisSetFailed)?;

    Ok(())
  }

  /// Get and deserialize a JSON-encoded value
  pub async fn get_value<T: serde::de::DeserializeOwned>(
    &mut self,
    key: &str,
  ) -> FastJobResult<Option<T>> {
    let value: Option<String> = self.connection
      .get(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    value
      .map(|v| serde_json::from_str(&v).with_fastjob_type(FastJobErrorType::DeserializationFailed))
      .transpose()
  }

  /// Delete a key
  pub async fn delete_key(&mut self, key: &str) -> FastJobResult<()> {
    let deleted: i64 = self.connection
      .del(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisDeleteFailed)?;

    if deleted == 0 {
      Err(FastJobErrorType::RedisKeyNotFound.into())
    } else {
      Ok(())
    }
  }
}
