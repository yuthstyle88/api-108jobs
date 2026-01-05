#![cfg(feature = "full")]
use crate::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use redis::aio::MultiplexedConnection;
pub use redis::AsyncCommands;
use serde_json;

#[derive(Clone)]
pub struct RedisClient {
  connection: MultiplexedConnection,
}

impl RedisClient {
  /// Initializes Redis client and connects immediately
  pub async fn new(connection: &str) -> FastJobResult<Self> {
    let client =
      redis::Client::open(connection).with_fastjob_type(FastJobErrorType::RedisConnectionFailed)?;

    let mut conn = client
      .get_multiplexed_async_connection()
      .await
      .with_fastjob_type(FastJobErrorType::RedisConnectionFailed)?;
    let _: String = redis::cmd("PING")
      .query_async(&mut conn)
      .await
      .with_fastjob_type(FastJobErrorType::RedisConnectionFailed)?;

    println!("Connected to Redis at {}", connection);

    Ok(Self { connection: conn })
  }

  /// Set a JSON-serialized value with expiration (unit second)
  pub async fn set_value_with_expiry<T: serde::Serialize>(
    &mut self,
    key: &str,
    value: T,
    expiry: usize,
  ) -> FastJobResult<()> {
    // ป้องกัน config ผิด (ทางเลือก)
    debug_assert!(expiry > 0, "expiry should be > 0");
    let value_str =
      serde_json::to_string(&value).with_fastjob_type(FastJobErrorType::SerializationFailed)?;

    let result: redis::RedisResult<()> =
      self.connection.set_ex(key, value_str, expiry as u64).await;

    result.map_err(|_| FastJobErrorType::RedisSetFailed)?;

    Ok(())
  }

  /// Get and deserialize a JSON-encoded value
  pub async fn get_value<T: serde::de::DeserializeOwned>(
    &mut self,
    key: &str,
  ) -> FastJobResult<Option<T>> {
    let value: Option<String> = self
      .connection
      .get(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    value
      .map(|v| serde_json::from_str(&v).with_fastjob_type(FastJobErrorType::DeserializationFailed))
      .transpose()
  }

  /// Delete a key
  pub async fn delete_key(&mut self, key: &str) -> FastJobResult<()> {
    let deleted: i64 = self
      .connection
      .del(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisDeleteFailed)?;

    if deleted == 0 {
      Err(FastJobErrorType::RedisKeyNotFound.into())
    } else {
      Ok(())
    }
  }

  /// Append a JSON-serialized value to a Redis list
  pub async fn rpush<T: serde::Serialize>(&mut self, key: &str, value: T) -> FastJobResult<()> {
    let value_str =
      serde_json::to_string(&value).with_fastjob_type(FastJobErrorType::SerializationFailed)?;
    let _: () = self
      .connection
      .rpush(key, value_str)
      .await
      .with_fastjob_type(FastJobErrorType::RedisSetFailed)?;
    Ok(())
  }

  /// Retrieve a range of values from a Redis list and deserialize them
  pub async fn lrange<T: serde::de::DeserializeOwned>(
    &mut self,
    key: &str,
    start: i64,
    stop: i64,
  ) -> FastJobResult<Vec<T>> {
    let start_isize = start.try_into().map_err(|_| {
      FastJobErrorType::InvalidInput("start index out of range for isize".to_string())
    })?;
    let stop_isize = stop.try_into().map_err(|_| {
      FastJobErrorType::InvalidInput("stop index out of range for isize".to_string())
    })?;
    let value_strs: Vec<String> = self
      .connection
      .lrange(key, start_isize, stop_isize)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    let mut values = Vec::new();
    for value_str in value_strs {
      let value = serde_json::from_str(&value_str)
        .with_fastjob_type(FastJobErrorType::DeserializationFailed)?;
      values.push(value);
    }
    Ok(values)
  }

  /// Add a value to a Redis set
  pub async fn sadd<T: ToString>(&mut self, key: &str, value: T) -> FastJobResult<()> {
    let _: () = self
      .connection
      .sadd(key, value.to_string())
      .await
      .with_fastjob_type(FastJobErrorType::RedisSetFailed)?;
    Ok(())
  }

  /// Remove a value from a Redis set
  pub async fn srem<T: ToString>(&mut self, key: &str, value: T) -> FastJobResult<()> {
    let removed: i64 = self
      .connection
      .srem(key, value.to_string())
      .await
      .with_fastjob_type(FastJobErrorType::RedisDeleteFailed)?;
    if removed == 0 {
      Err(FastJobErrorType::RedisKeyNotFound.into())
    } else {
      Ok(())
    }
  }

  /// Retrieve all members of a Redis set
  pub async fn smembers(&mut self, key: &str) -> FastJobResult<Vec<String>> {
    let value_strs: Vec<String> = self
      .connection
      .smembers(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    Ok(value_strs)
  }

  /// Set expiration time on a key
  pub async fn expire(&mut self, key: &str, seconds: usize) -> FastJobResult<()> {
    let _: () = self
      .connection
      .expire(key, seconds as i64)
      .await
      .with_fastjob_type(FastJobErrorType::RedisSetFailed)?;
    Ok(())
  }

  /// Get keys matching a pattern
  pub async fn keys(&mut self, pattern: &str) -> FastJobResult<Vec<String>> {
    let keys: Vec<String> = self
      .connection
      .keys(pattern)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    Ok(keys)
  }
}
