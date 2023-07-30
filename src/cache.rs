//! Defintion and implementations of the [Cache] trait.

use std::{collections::HashMap, fmt::Debug, num::TryFromIntError};

use crate::result::{Error, Result};
use async_trait::async_trait;
use reool::{AsyncCommands, PoolDefault, RedisPool};
use serde::{de::DeserializeOwned, Serialize};

/// Represents a general purpose cache.
#[async_trait]
pub trait Cache {
    async fn find<T: DeserializeOwned>(&self, key: &str) -> Result<T>;
    async fn save<T: Serialize + Send + Sync + Debug>(
        &self,
        key: &str,
        value: T,
        expire: Option<u64>,
    ) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

/// Redis implementation of [`Cache`].
pub struct RedisCache<'a> {
    pub pool: &'a RedisPool,
}

#[async_trait]
impl<'a> Cache for RedisCache<'a> {
    #[instrument(skip(self))]
    async fn find<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!(error = err.to_string(), "pulling connection for redis",);
            Error::Unknown
        })?;

        let data: String = conn.get(key).await.map_err(|err| {
            error!(error = err.to_string(), "performing GET command on redis",);
            Error::Unknown
        })?;

        serde_json::from_str(&data).map_err(|err| {
            error!(error = err.to_string(), "deserializing data of type T",);
            Error::Unknown
        })
    }

    #[instrument(skip(self))]
    async fn save<T: Serialize + Send + Sync + Debug>(
        &self,
        key: &str,
        value: T,
        expire: Option<u64>,
    ) -> Result<()> {
        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!(error = err.to_string(), "pulling connection for redis",);
            Error::Unknown
        })?;

        let data = serde_json::to_string(&value).map_err(|err| {
            error!(error = err.to_string(), "serializing data of type T",);
            Error::Unknown
        })?;

        conn.set(key, data).await.map_err(|err| {
            error!(error = err.to_string(), "performing SET command on redis",);
            Error::Unknown
        })?;

        if let Some(expire) = expire {
            let expire = expire.try_into().map_err(|err: TryFromIntError| {
                error!(error = err.to_string(), "parsing expiration time to usize",);
                Error::Unknown
            })?;

            conn.expire(key, expire).await.map_err(|err| {
                error!(
                    error = err.to_string(),
                    "performing EXPIRE command on redis",
                );
                Error::Unknown
            })?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!(error = err.to_string(), "pulling connection for redis",);
            Error::Unknown
        })?;

        conn.del(key).await.map_err(|err| {
            error!(
                error = err.to_string(),
                "performing DELETE command on redis",
            );
            Error::Unknown
        })?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::Cache;
    use crate::result::{Error, Result};
    use async_trait::async_trait;
    use once_cell::sync::Lazy;
    use serde::{de::DeserializeOwned, Serialize};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    pub static IN_MEMORY_CACHE: Lazy<Arc<Mutex<HashMap<String, String>>>> =
        Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

    #[derive(Default)]
    pub struct InMemoryCache;

    #[async_trait]
    impl Cache for InMemoryCache {
        async fn find<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
            let data: String = IN_MEMORY_CACHE
                .lock()
                .unwrap()
                .get(key)
                .map(ToString::to_string)
                .ok_or(Error::NotFound)?;

            serde_json::from_str(&data).map_err(|err| {
                error!(error = err.to_string(), "deserializing data of type T",);
                Error::Unknown
            })
        }

        async fn save<T: Serialize + Send + Sync>(
            &self,
            key: &str,
            value: T,
            expire: Option<u64>,
        ) -> Result<()> {
            let data = serde_json::to_string(&value).map_err(|err| {
                error!(error = err.to_string(), "serializing data of type T",);
                Error::Unknown
            })?;

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
