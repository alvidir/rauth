//! Defintion and implementations of the [Cache] trait.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::time::Duration;

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<Error> for Result<T> {
    fn from(error: Error) -> Self {
        Err(error)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("resource not found")]
    NotFound,
    #[error("{0}")]
    Expiration(#[from] std::num::TryFromIntError),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "redis-cache")]
    #[error("{0}")]
    Redis(#[from] reool::RedisError),
    #[cfg(feature = "redis-cache")]
    #[error("{0}")]
    Checkout(#[from] reool::CheckoutError),
}

/// Represents a general purpose cache.
#[async_trait]
pub trait Cache {
    async fn find<T>(&self, key: &str) -> Result<T>
    where
        T: DeserializeOwned;
    async fn save<T>(&self, key: &str, value: T, expire: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync + Debug;
    async fn delete(&self, key: &str) -> Result<()>;
}

#[cfg(feature = "redis-cache")]
pub use redis_cache::*;

#[cfg(feature = "redis-cache")]
mod redis_cache {
    use super::{Cache, Error, Result};
    use crate::on_error;
    use async_trait::async_trait;
    use reool::{AsyncCommands, PoolDefault, RedisError, RedisPool};
    use serde::{de::DeserializeOwned, Serialize};
    use std::fmt::Debug;
    use std::num::TryFromIntError;
    use std::time::Duration;

    /// Redis implementation of [`Cache`].
    pub struct RedisCache<'a> {
        pub pool: &'a RedisPool,
    }

    #[async_trait]
    impl<'a> Cache for RedisCache<'a> {
        #[instrument(skip(self))]
        async fn find<T>(&self, key: &str) -> Result<T>
        where
            T: DeserializeOwned,
        {
            let mut conn = self
                .pool
                .check_out(PoolDefault)
                .await
                .map_err(on_error!(Error, "pulling connection for redis"))?;

            let Some(data): Option<String> = conn
                .get(key)
                .await
                .map_err(on_error!(Error, "performing GET command on redis"))?
            else {
                return Error::NotFound.into();
            };

            serde_json::from_str(&data).map_err(on_error!(Error, "deserializing data from redis"))
        }

        #[instrument(skip(self))]
        async fn save<T>(&self, key: &str, value: T, expire: Option<Duration>) -> Result<()>
        where
            T: Serialize + Send + Sync + Debug,
        {
            let mut conn = self
                .pool
                .check_out(PoolDefault)
                .await
                .map_err(on_error!(Error, "pulling connection for redis"))?;

            let data = serde_json::to_string(&value)
                .map_err(on_error!(Error, "serializing data for redis"))?;

            conn.set(key, data)
                .await
                .map_err(on_error!(Error, "performing SET command on redis"))?;

            if let Some(expire) = expire {
                let expire = expire.as_secs().try_into().map_err(on_error!(
                    TryFromIntError as Error,
                    "parsing expiration time to usize"
                ))?;

                conn.expire(key, expire)
                    .await
                    .map_err(on_error!(Error, "performing EXPIRE command on redis"))?;
            }

            Ok(())
        }

        #[instrument(skip(self))]
        async fn delete(&self, key: &str) -> Result<()> {
            let mut conn = self
                .pool
                .check_out(PoolDefault)
                .await
                .map_err(RedisError::from)
                .map_err(on_error!(Error, "pulling connection for redis"))?;

            conn.del(key)
                .await
                .map_err(RedisError::from)
                .map_err(on_error!(Error, "performing DELETE command on redis"))?;

            Ok(())
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Cache, Error, Result};
    use crate::on_error;
    use async_trait::async_trait;
    use once_cell::sync::Lazy;
    use serde::{de::DeserializeOwned, Serialize};
    use std::fmt::Debug;
    use std::time::Duration;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    pub static IN_MEMORY_CACHE: Lazy<Arc<Mutex<HashMap<String, String>>>> =
        Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

    /// In memory implementation of [`Cache`].
    #[derive(Default)]
    pub struct InMemoryCache;

    #[async_trait]
    impl Cache for InMemoryCache {
        async fn find<T>(&self, key: &str) -> Result<T>
        where
            T: DeserializeOwned,
        {
            let Some(data) = IN_MEMORY_CACHE
                .lock()
                .unwrap()
                .get(key)
                .map(ToString::to_string)
            else {
                return Error::NotFound.into();
            };

            serde_json::from_str(&data).map_err(on_error!(Error, "deserializing data from json"))
        }

        async fn save<T>(&self, key: &str, value: T, expire: Option<Duration>) -> Result<()>
        where
            T: Serialize + Send + Sync + Debug,
        {
            let data = serde_json::to_string(&value)
                .map_err(on_error!(Error, "serializing data to json"))?;

            IN_MEMORY_CACHE
                .lock()
                .unwrap()
                .insert(key.to_string(), data);

            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<()> {
            IN_MEMORY_CACHE.lock().unwrap().remove(key);
            Ok(())
        }
    }
}
