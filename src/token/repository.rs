use super::application::TokenRepository;
use crate::result::{Error, Result};
use async_trait::async_trait;
use reool::AsyncCommands;
use reool::*;

pub struct RedisTokenRepository<'a> {
    pub pool: &'a RedisPool,
    pub jwt_secret: &'a [u8],
    pub jwt_public: &'a [u8],
}

#[async_trait]
impl<'a> TokenRepository for RedisTokenRepository<'a> {
    async fn find(&self, key: &str) -> Result<String> {
        info!("looking for token with id {}", key);

        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!("{} pulling connection for redis: {}", Error::Unknown, err);
            Error::Unknown
        })?;

        let token: Vec<u8> = conn.get(key).await.map_err(|err| {
            error!(
                "{} performing GET command on redis: {}",
                Error::Unknown,
                err
            );
            Error::Unknown
        })?;

        let token: String = String::from_utf8(token).map_err(|err| {
            error!("{} parsing token to string: {}", Error::Unknown, err);
            Error::Unknown
        })?;
        Ok(token)
    }

    async fn save(&self, key: &str, token: &str, expire: Option<u64>) -> Result<()> {
        info!("storing token with id {}", key);

        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!("{} pulling connection for redis: {}", Error::Unknown, err);
            Error::Unknown
        })?;

        conn.set(key, token).await.map_err(|err| {
            error!(
                "{} performing SET command on redis: {}",
                Error::Unknown,
                err
            );
            Error::Unknown
        })?;

        if let Some(expire) = expire {
            let expire = expire.try_into().map_err(|err| {
                error!(
                    "{} parsing expiration time to usize: {}",
                    Error::Unknown,
                    err
                );
                Error::Unknown
            })?;

            conn.expire(key, expire).await.map_err(|err| {
                error!(
                    "{} performing EXPIRE command on redis: {}",
                    Error::Unknown,
                    err
                );
                Error::Unknown
            })?;
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        info!("removing token with id {}", key);

        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!("{} pulling connection for redis: {}", Error::Unknown, err);
            Error::Unknown
        })?;

        conn.del(key).await.map_err(|err| {
            error!(
                "{} performing DELETE command on redis: {}",
                Error::Unknown,
                err
            );
            Error::Unknown
        })?;

        Ok(())
    }
}
