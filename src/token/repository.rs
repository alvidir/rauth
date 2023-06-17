use std::num::TryFromIntError;

use super::application::TokenRepository;
use crate::result::{Error, Result};
use async_trait::async_trait;
use reool::AsyncCommands;
use reool::*;

pub struct RedisTokenRepository<'a> {
    pub pool: &'a RedisPool,
}

#[async_trait]
impl<'a> TokenRepository for RedisTokenRepository<'a> {
    #[instrument(skip(self))]
    async fn find(&self, key: &str) -> Result<String> {
        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!(error = err.to_string(), "pulling connection for redis",);
            Error::Unknown
        })?;

        let token: Vec<u8> = conn.get(key).await.map_err(|err| {
            error!(error = err.to_string(), "performing GET command on redis",);
            Error::Unknown
        })?;

        let token: String = String::from_utf8(token).map_err(|err| {
            error!(error = err.to_string(), "parsing token to string",);
            Error::Unknown
        })?;
        Ok(token)
    }

    #[instrument(skip(self))]
    async fn save(&self, key: &str, token: &str, expire: Option<u64>) -> Result<()> {
        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!(error = err.to_string(), "pulling connection for redis",);
            Error::Unknown
        })?;

        conn.set(key, token).await.map_err(|err| {
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
