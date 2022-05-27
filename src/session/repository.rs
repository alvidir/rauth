use super::application::TokenRepository;
use crate::constants;
use async_trait::async_trait;
use reool::AsyncCommands;
use reool::*;
use std::error::Error;

pub struct RedisTokenRepository<'a> {
    pub pool: &'a RedisPool,
    pub jwt_secret: &'a [u8],
    pub jwt_public: &'a [u8],
}

#[async_trait]
impl<'a> TokenRepository for RedisTokenRepository<'a> {
    async fn find(&self, key: &str) -> Result<String, Box<dyn Error>> {
        info!("looking for token with id {}", key);

        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!(
                "{} pulling connection for redis: {}",
                constants::ERR_UNKNOWN,
                err
            );
            constants::ERR_UNKNOWN
        })?;

        let token: Vec<u8> = conn.get(key).await.map_err(|err| {
            error!(
                "{} performing GET command on redis: {}",
                constants::ERR_UNKNOWN,
                err
            );
            constants::ERR_UNKNOWN
        })?;

        let token: String = String::from_utf8(token).map_err(|err| {
            error!(
                "{} parsing token to string: {}",
                constants::ERR_UNKNOWN,
                err
            );
            constants::ERR_UNKNOWN
        })?;
        Ok(token)
    }

    async fn save(
        &self,
        key: &str,
        token: &str,
        expire: Option<u64>,
    ) -> Result<(), Box<dyn Error>> {
        info!("storing token with id {}", key);

        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!(
                "{} pulling connection for redis: {}",
                constants::ERR_UNKNOWN,
                err
            );
            constants::ERR_UNKNOWN
        })?;

        conn.set(key, token).await.map_err(|err| {
            error!(
                "{} performing SET command on redis: {}",
                constants::ERR_UNKNOWN,
                err
            );
            constants::ERR_UNKNOWN
        })?;

        if let Some(expire) = expire {
            let expire = expire.try_into().map_err(|err| {
                error!(
                    "{} parsing expiration time to usize: {}",
                    constants::ERR_UNKNOWN,
                    err
                );
                constants::ERR_UNKNOWN
            })?;

            conn.expire(key, expire).await.map_err(|err| {
                error!(
                    "{} performing EXPIRE command on redis: {}",
                    constants::ERR_UNKNOWN,
                    err
                );
                constants::ERR_UNKNOWN
            })?;
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
        info!("removing token with id {}", key);

        let mut conn = self.pool.check_out(PoolDefault).await.map_err(|err| {
            error!(
                "{} pulling connection for redis: {}",
                constants::ERR_UNKNOWN,
                err
            );
            constants::ERR_UNKNOWN
        })?;

        conn.del(key).await.map_err(|err| {
            error!(
                "{} performing DELETE command on redis: {}",
                constants::ERR_UNKNOWN,
                err
            );
            constants::ERR_UNKNOWN
        })?;

        Ok(())
    }
}
