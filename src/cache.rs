//! Defintion and implementations of the [Cache] trait.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::time::Duration;

/// Represents a general purpose cache.
#[async_trait]
pub trait Cache {
    async fn find<T, Err>(&self, key: &str) -> Result<Option<T>, Err>
    where
        T: DeserializeOwned,
        Err: From<String>;
    async fn save<T, Err>(&self, key: &str, value: T, expire: Option<Duration>) -> Result<(), Err>
    where
        T: Serialize + Send + Sync + Debug,
        Err: From<String>;
    async fn delete<Err>(&self, key: &str) -> Result<(), Err>
    where
        Err: From<String>;
}

#[cfg(feature = "redis-cache")]
pub use redis_cache::*;

#[cfg(feature = "redis-cache")]
mod redis_cache {
    use super::Cache;
    use crate::on_error;
    use async_trait::async_trait;
    use reool::{AsyncCommands, PoolDefault, RedisPool};
    use serde::{de::DeserializeOwned, Serialize};
    use std::fmt::Debug;
    use std::time::Duration;

    /// Redis implementation of [`Cache`].
    pub struct RedisCache<'a> {
        pub pool: &'a RedisPool,
    }

    #[async_trait]
    impl<'a> Cache for RedisCache<'a> {
        #[instrument(skip(self))]
        async fn find<T, Err>(&self, key: &str) -> Result<Option<T>, Err>
        where
            T: DeserializeOwned,
            Err: From<String>,
        {
            let mut conn = self
                .pool
                .check_out(PoolDefault)
                .await
                .map_err(on_error!("pulling connection for redis"))?;

            let Some(data): Option<String> = conn
                .get(key)
                .await
                .map_err(on_error!("performing GET command on redis"))?
            else {
                return Ok(None);
            };

            serde_json::from_str(&data).map_err(on_error!("deserializing data from redis"))
        }

        #[instrument(skip(self))]
        async fn save<T, Err>(
            &self,
            key: &str,
            value: T,
            expire: Option<Duration>,
        ) -> Result<(), Err>
        where
            T: Serialize + Send + Sync + Debug,
            Err: From<String>,
        {
            let mut conn = self
                .pool
                .check_out(PoolDefault)
                .await
                .map_err(on_error!("pulling connection for redis"))?;

            let data =
                serde_json::to_string(&value).map_err(on_error!("serializing data for redis"))?;

            conn.set(key, data)
                .await
                .map_err(on_error!("performing SET command on redis"))?;

            if let Some(expire) = expire {
                let expire = expire
                    .as_secs()
                    .try_into()
                    .map_err(on_error!("parsing expiration time to usize"))?;

                conn.expire(key, expire)
                    .await
                    .map_err(on_error!("performing EXPIRE command on redis"))?;
            }

            Ok(())
        }

        #[instrument(skip(self))]
        async fn delete<Err>(&self, key: &str) -> Result<(), Err>
        where
            Err: From<String>,
        {
            let mut conn = self
                .pool
                .check_out(PoolDefault)
                .await
                .map_err(on_error!("pulling connection for redis"))?;

            conn.del(key)
                .await
                .map_err(on_error!("performing DELETE command on redis"))?;

            Ok(())
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::Cache;
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
        async fn find<T, Err>(&self, key: &str) -> Result<Option<T>, Err>
        where
            T: DeserializeOwned,
            Err: From<String>,
        {
            let Some(data) = IN_MEMORY_CACHE
                .lock()
                .unwrap()
                .get(key)
                .map(ToString::to_string)
            else {
                return Ok(None);
            };

            serde_json::from_str(&data).map_err(on_error!("deserializing data from json"))
        }

        async fn save<T, Err>(
            &self,
            key: &str,
            value: T,
            expire: Option<Duration>,
        ) -> Result<(), Err>
        where
            T: Serialize + Send + Sync + Debug,
            Err: From<String>,
        {
            let data =
                serde_json::to_string(&value).map_err(on_error!("serializing data to json"))?;

            IN_MEMORY_CACHE
                .lock()
                .unwrap()
                .insert(key.to_string(), data);

            Ok(())
        }

        async fn delete<Err>(&self, key: &str) -> Result<(), Err>
        where
            Err: From<String>,
        {
            IN_MEMORY_CACHE.lock().unwrap().remove(key);
            Ok(())
        }
    }
}
