use std::error::Error;
use std::ops::DerefMut;
use r2d2_redis::{r2d2, redis, RedisConnectionManager};
use super::application::SessionRepository;
use crate::security;

const REDIS_CMD_GET: &str = "GET";
const REDIS_CMD_SET: &str = "SET";
const REDIS_CMD_DELETE: &str = "DEL";

type RdPool = r2d2::Pool<RedisConnectionManager> ;

pub struct RedisSessionRepository<'a> {
    pub pool: &'a RdPool,
    pub rsa_public: &'a [u8],
    pub jwt_secret: &'a [u8],
    pub jwt_public: &'a [u8],
}

impl<'a> SessionRepository for RedisSessionRepository<'a> {
    fn exists(&self, key: &str) -> Result<(), Box<dyn Error>> {
        info!("looking for token with key {}", key);

        let mut conn = self.pool.get()?;
        redis::cmd(REDIS_CMD_GET).arg(key).query(conn.deref_mut())?;
        Ok(())
    }

    fn save(&self, key: &str, token: &str) -> Result<(), Box<dyn Error>> {
        info!("storing token with key {} and value {}", key, token);
        
        let mut conn = self.pool.get()?;
        let secure_token = security::encrypt(self.rsa_public, token.as_bytes())?;
        redis::cmd(REDIS_CMD_SET).arg(key).arg(secure_token).query(conn.deref_mut())?;
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
        info!("removing token with key {}", key);

        let mut conn = self.pool.get()?;
        redis::cmd(REDIS_CMD_DELETE).arg(key).query(conn.deref_mut())?;
        Ok(())
    }

}