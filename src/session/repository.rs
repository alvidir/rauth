use redis;
use std::error::Error;
use crate::security;

use super::{
    application::SessionRepository,
    domain::SessionToken,
};

pub struct RedisSessionRepository {
    pub conn: redis::Connection,
    pub jwt_secret: Vec<u8>,
    pub jwt_public: Vec<u8>,
}

impl SessionRepository for RedisSessionRepository {
    fn find(&mut self, token: &SessionToken) -> Result<SessionToken, Box<dyn Error>> {
        let secure_token: String = redis::cmd("GET").arg(token.sub).query(&mut self.conn)?;
        let token = security::decode_jwt(&self.jwt_public, &secure_token)?;
        Ok(token)
    }

    fn save(&mut self, token: &SessionToken) -> Result<(), Box<dyn Error>> {
        let secure_token = security::encode_jwt(&self.jwt_secret, token)?;
        redis::cmd("SET").arg(token.sub).arg(secure_token).query(&mut self.conn)?;
        Ok(())
    }

    fn delete(&mut self, token: &SessionToken) -> Result<(), Box<dyn Error>> {
        redis::cmd("DEL").arg(token.sub).query(&mut self.conn)?;
        Ok(())
    }

}