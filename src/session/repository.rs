use std::error::Error;
use std::ops::DerefMut;
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use r2d2_redis::{r2d2, redis, RedisConnectionManager};
use crate::security;
use super::application::SessionRepository;

const REDIS_CMD_GET: &str = "GET";
const REDIS_CMD_SET: &str = "SET";
const REDIS_CMD_DELETE: &str = "DEL";

type RdPool = r2d2::Pool<RedisConnectionManager> ;

pub struct RedisSessionRepository<'a> {
    pub pool: &'a RdPool,
    pub jwt_secret: &'a [u8],
    pub jwt_public: &'a [u8],
}

impl<'a> SessionRepository for RedisSessionRepository<'a> {
    fn exists(&self, key: u64) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let secure_token: String = redis::cmd(REDIS_CMD_GET).arg(key).query(conn.deref_mut())?;
        security::verify_jwt(&self.jwt_public, &secure_token)?;
        Ok(())
    }

    fn save<T: Serialize + DeserializeOwned>(&self, key: u64, token: &T) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let secure_token = security::sign_jwt(self.jwt_secret, token)?;
        redis::cmd(REDIS_CMD_SET).arg(key).arg(secure_token).query(conn.deref_mut())?;
        Ok(())
    }

    fn delete(&self, key: u64) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        redis::cmd(REDIS_CMD_DELETE).arg(key).query(conn.deref_mut())?;
        Ok(())
    }

}