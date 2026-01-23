#![cfg(feature = "full")]
use crate::error::{FastJobErrorExt, FastJobErrorType, FastJobResult};
use redis::aio::MultiplexedConnection;
pub use redis::AsyncCommands;
use redis::{Pipeline, Value};
use serde_json;

#[derive(Clone)]
pub struct RedisClient {
  pub connection: MultiplexedConnection,
}

impl RedisClient {
  pub fn pipeline(&mut self) -> Pipeline {
    redis::pipe()
  }

  /// Execute pipeline and return raw Redis values (one per command)
  pub async fn exec_pipeline(&mut self, pipe: &mut Pipeline) -> FastJobResult<Vec<Value>> {
    let results: Vec<Value> = pipe
      .query_async(&mut self.connection)
      .await
      .with_fastjob_type(FastJobErrorType::RedisPipelineFailed)?;
    Ok(results)
  }

  pub async fn new(connection: &str) -> FastJobResult<Self> {
    let client =
      redis::Client::open(connection).with_fastjob_type(FastJobErrorType::RedisConnectionFailed)?;

    let mut conn = client
      .get_multiplexed_async_connection()
      .await
      .with_fastjob_type(FastJobErrorType::RedisConnectionFailed)?;

    let pong: String = redis::cmd("PING")
      .query_async(&mut conn)
      .await
      .with_fastjob_type(FastJobErrorType::RedisConnectionFailed)?;

    if pong != "PONG" {
      return Err(FastJobErrorType::RedisConnectionFailed.into());
    }

    tracing::info!("Connected to Redis at {}", connection);

    Ok(Self { connection: conn })
  }

  // === Basic Operations ===

  /// Publish a text payload to a Redis pub/sub channel
  pub async fn publish(&mut self, channel: &str, payload: &str) -> FastJobResult<()> {
    let _: () = redis::cmd("PUBLISH")
      .arg(channel)
      .arg(payload)
      .query_async(&mut self.connection)
      .await
      .with_fastjob_type(FastJobErrorType::RedisSetFailed)?;
    Ok(())
  }

  pub async fn set_value_with_expiry<T: serde::Serialize>(
    &mut self,
    key: &str,
    value: T,
    expiry: usize,
  ) -> FastJobResult<()> {
    let value_str =
      serde_json::to_string(&value).with_fastjob_type(FastJobErrorType::SerializationFailed)?;

    let _: () = self
      .connection
      .set_ex(key, value_str, expiry as u64)
      .await
      .with_fastjob_type(FastJobErrorType::RedisSetFailed)?;
    Ok(())
  }

  pub async fn get_value<T: serde::de::DeserializeOwned>(
    &mut self,
    key: &str,
  ) -> FastJobResult<Option<T>> {
    let value: Option<String> = self
      .connection
      .get(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;

    match value {
      Some(v) => {
        let deserialized =
          serde_json::from_str(&v).with_fastjob_type(FastJobErrorType::DeserializationFailed)?;
        Ok(Some(deserialized))
      }
      None => Ok(None),
    }
  }

  // Fire-and-forget delete (don't error if key missing)
  pub async fn delete_key(&mut self, key: &str) -> FastJobResult<()> {
    let _: i64 = self
      .connection
      .del(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisDeleteFailed)?;
    Ok(())
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

  // === Set Operations ===

  pub async fn sadd<T: ToString>(&mut self, key: &str, value: T) -> FastJobResult<i64> {
    let added: i64 = self
      .connection
      .sadd(key, value.to_string())
      .await
      .with_fastjob_type(FastJobErrorType::RedisSetFailed)?;
    Ok(added)
  }

  pub async fn srem<T: ToString>(&mut self, key: &str, value: T) -> FastJobResult<i64> {
    let removed: i64 = self
      .connection
      .srem(key, value.to_string())
      .await
      .with_fastjob_type(FastJobErrorType::RedisDeleteFailed)?;
    Ok(removed)
  }

  pub async fn smembers(&mut self, key: &str) -> FastJobResult<Vec<String>> {
    let members: Vec<String> = self
      .connection
      .smembers(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    Ok(members)
  }

  pub async fn sismember<T: ToString>(&mut self, key: &str, member: T) -> FastJobResult<bool> {
    let exists: bool = self
      .connection
      .sismember(key, member.to_string())
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    Ok(exists)
  }

  pub async fn scard(&mut self, key: &str) -> FastJobResult<usize> {
    let count: i64 = self
      .connection
      .scard(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    Ok(count as usize)
  }

  // === Key Operations ===

  pub async fn expire(&mut self, key: &str, seconds: usize) -> FastJobResult<bool> {
    let ok: bool = self
      .connection
      .expire(key, seconds as i64)
      .await
      .with_fastjob_type(FastJobErrorType::RedisSetFailed)?;
    Ok(ok)
  }

  pub async fn ttl(&mut self, key: &str) -> FastJobResult<Option<i64>> {
    let ttl: i64 = self
      .connection
      .ttl(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    Ok((ttl > 0).then(|| ttl))
  }

  pub async fn keys(&mut self, pattern: &str) -> FastJobResult<Vec<String>> {
    let keys: Vec<String> = self
      .connection
      .keys(pattern)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    Ok(keys)
  }

  pub async fn exists(&mut self, key: &str) -> FastJobResult<bool> {
    let exists: bool = self
      .connection
      .exists(key)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    Ok(exists)
  }

  // === SCAN support for safe iteration ===
  pub async fn scan(
    &mut self,
    cursor: u64,
    pattern: &str,
    count: usize,
  ) -> FastJobResult<(u64, Vec<String>)> {
    let result: (u64, Vec<String>) = redis::cmd("SCAN")
      .arg(cursor)
      .arg("MATCH")
      .arg(pattern)
      .arg("COUNT")
      .arg(count)
      .query_async(&mut self.connection)
      .await
      .with_fastjob_type(FastJobErrorType::RedisGetFailed)?;
    Ok(result)
  }
}
