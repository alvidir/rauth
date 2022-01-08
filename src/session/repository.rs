use std::error::Error;
use std::ops::DerefMut;
use r2d2_redis::{r2d2, redis, RedisConnectionManager};
use crate::security;

const REDIS_CMD_GET: &str = "GET";
const REDIS_CMD_SET: &str = "SET";
const REDIS_CMD_DELETE: &str = "DEL";

use super::{
    application::SessionRepository,
    domain::SessionToken,
};

type RdPool = r2d2::Pool<RedisConnectionManager> ;

pub struct RedisSessionRepository<'a> {
    pub pool: &'a RdPool,
    pub jwt_secret: &'static [u8],
    pub jwt_public: &'static [u8],
}

impl<'a> SessionRepository for RedisSessionRepository<'a> {
    fn find(&self, user_id: i32) -> Result<SessionToken, Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let secure_token: String = redis::cmd(REDIS_CMD_GET).arg(user_id).query(conn.deref_mut())?;
        let token = security::decode_jwt(&self.jwt_public, &secure_token)?;
        Ok(token)
    }

    fn save(&self, token: &SessionToken) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        let secure_token = security::encode_jwt(&self.jwt_secret, token)?;
        redis::cmd(REDIS_CMD_SET).arg(token.sub).arg(secure_token).query(conn.deref_mut())?;
        Ok(())
    }

    fn delete(&self, user_id: i32) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get()?;
        redis::cmd(REDIS_CMD_DELETE).arg(user_id).query(conn.deref_mut())?;
        Ok(())
    }

}