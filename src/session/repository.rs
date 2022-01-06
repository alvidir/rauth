use redis;
use std::error::Error;
use crate::security;

use super::{
    application::SessionRepository,
    domain::SessionToken,
};

pub struct RedisSessionRepository {
    pub conn: fn() -> Result<redis::Connection, Box<dyn Error>>,
    pub jwt_secret: &'static [u8],
    pub jwt_public: &'static [u8],
}

impl SessionRepository for RedisSessionRepository {
    fn find(&self, user_id: i32) -> Result<SessionToken, Box<dyn Error>> {
        let secure_token: String = redis::cmd("GET").arg(user_id).query(&mut (self.conn)()?)?;
        let token = security::decode_jwt(&self.jwt_public, &secure_token)?;
        Ok(token)
    }

    fn save(&self, token: &SessionToken) -> Result<(), Box<dyn Error>> {
        let secure_token = security::encode_jwt(&self.jwt_secret, token)?;
        redis::cmd("SET").arg(token.sub).arg(secure_token).query(&mut (self.conn)()?)?;
        Ok(())
    }

    fn delete(&self, user_id: i32) -> Result<(), Box<dyn Error>> {
        redis::cmd("DEL").arg(user_id).query(&mut (self.conn)()?)?;
        Ok(())
    }

}