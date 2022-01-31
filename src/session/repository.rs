use std::error::Error;
use std::ops::DerefMut;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

impl<'a> RedisSessionRepository<'a> {
    fn hash<T: Serialize + DeserializeOwned + Hash>(token: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        hasher.finish()
    }
}

impl<'a> SessionRepository for RedisSessionRepository<'a> {
    fn find<T: Serialize + DeserializeOwned + Hash>(&self, token: &T) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let key = RedisSessionRepository::hash(token);
        let secure_token: String = redis::cmd(REDIS_CMD_GET).arg(key).query(conn.deref_mut())?;
        security::verify_jwt::<T>(&self.jwt_public, &secure_token)?;
        Ok(())
    }

    fn save<T: Serialize + DeserializeOwned + Hash>(&self, token: &T) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let key = RedisSessionRepository::hash(token);
        let secure_token = security::sign_jwt(self.jwt_secret, token)?;
        redis::cmd(REDIS_CMD_SET).arg(key).arg(secure_token).query(conn.deref_mut())?;
        Ok(())
    }

    fn delete<T: Serialize + DeserializeOwned + Hash>(&self, token: &T) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let key = RedisSessionRepository::hash(token);
        redis::cmd(REDIS_CMD_DELETE).arg(key).query(conn.deref_mut())?;
        Ok(())
    }

}