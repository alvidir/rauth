use std::error::Error;
use std::ops::DerefMut;
use r2d2_redis::{r2d2, redis, RedisConnectionManager};
use super::application::TokenRepository;
use crate::constants;

const REDIS_CMD_GET: &str = "GET";
const REDIS_CMD_SET: &str = "SET";
const REDIS_CMD_DELETE: &str = "DEL";
const REDIS_CMD_EXPIRE: &str = "EXPIRE";

type RdPool = r2d2::Pool<RedisConnectionManager> ;

pub struct RedisTokenRepository<'a> {
    pub pool: &'a RdPool,
    pub jwt_secret: &'a [u8],
    pub jwt_public: &'a [u8],
}

impl<'a> TokenRepository for RedisTokenRepository<'a> {
    fn find(&self, key: &str) -> Result<String, Box<dyn Error>> {
        info!("looking for token with id {}", key);

        let mut conn = self.pool.get()
            .map_err(|err| {
                error!("{} pulling connection for redis: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;
        
        let token: String = redis::cmd(REDIS_CMD_GET).arg(key).query(conn.deref_mut())
            .map_err(|err| {
                error!("{} performing GET command on redis: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;
       
        Ok(token)
    }

    fn save(&self, key: &str, token: &str, expire: Option<u64>) -> Result<(), Box<dyn Error>> {
        info!("storing token with id {}", key);
        
        let mut conn = self.pool.get()
            .map_err(|err| {
                error!("{} pulling connection for redis: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;

        redis::cmd(REDIS_CMD_SET).arg(key).arg(token).query(conn.deref_mut())
            .map_err(|err| {
                error!("{} performing SET command on redis: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;

        if let Some(expire) = expire {
            redis::cmd(REDIS_CMD_EXPIRE).arg(key).arg(expire).query(conn.deref_mut())
                .map_err(|err| {
                    error!("{} performing EXPIRE command on redis: {}", constants::ERR_UNKNOWN, err);
                    constants::ERR_UNKNOWN
                })?;
        }
        
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
        info!("removing token with id {}", key);

        let mut conn = self.pool.get()
            .map_err(|err| {
                error!("{} pulling connection for redis: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;
        
        redis::cmd(REDIS_CMD_DELETE).arg(key).query(conn.deref_mut())
            .map_err(|err| {
                error!("{} performing DELETE command on redis: {}", constants::ERR_UNKNOWN, err);
                constants::ERR_UNKNOWN
            })?;
        
            Ok(())
    }

}