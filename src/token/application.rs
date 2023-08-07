use super::domain::{Token, TokenKind};
use crate::cache::Cache;
use crate::crypto;
use crate::result::Result;
use std::sync::Arc;
use std::time::Duration;

pub struct TokenApplication<'a, C: Cache> {
    pub timeout: Duration,
    pub token_issuer: &'a str,
    pub private_key: &'a [u8],
    pub public_key: &'a [u8],
    pub cache: Arc<C>,
}

impl<'a, T: Cache> TokenApplication<'a, T> {
    /// Returns a new token with the given kind and subject.
    #[instrument(skip(self))]
    pub fn generate(&self, kind: TokenKind, sub: &str) -> Result<Token> {
        Ok(Token::new(kind, self.token_issuer, sub, self.timeout))
    }

    /// Returns the resulting string of signing the given token.
    #[instrument(skip(self))]
    pub fn sign(&self, token: &Token) -> Result<String> {
        crypto::sign_jwt(self.private_key, token)
    }

    /// Stores the given token in the cache for a limited amount of time.
    #[instrument(skip(self))]
    pub async fn store(&self, token: &Token) -> Result<()> {
        self.cache
            .save(&token.jti, token, Some(token.timeout()))
            .await
    }

    /// Returns the paylod of the given JWT string.
    #[instrument(skip(self))]
    pub fn decode(&self, token: &str) -> Result<Token> {
        crypto::decode_jwt(self.public_key, token)
    }

    /// Retrives the token associated to the given key, if any.
    #[instrument(skip(self))]
    pub async fn find(&self, jti: &str) -> Result<Token> {
        self.cache.find(jti).await
    }

    /// Removes the token with the given ID from the cache, making it invalid.
    #[instrument(skip(self))]
    pub async fn revoke(&self, jti: &str) -> Result<()> {
        self.cache.delete(jti).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::TokenApplication;
    use crate::cache::tests::InMemoryCache;
    use crate::token::domain::{Token, TokenKind};
    use crate::{crypto, result::Error};
    use base64::{engine::general_purpose, Engine as _};
    use once_cell::sync::Lazy;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    pub static PRIVATE_KEY: Lazy<Vec<u8>> = Lazy::new(|| {
        general_purpose::STANDARD.decode(
            b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg=="
        ).unwrap()
    });

    pub static PUBLIC_KEY: Lazy<Vec<u8>> = Lazy::new(|| {
        general_purpose::STANDARD.decode(
            b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg=="
        ).unwrap()
    });

    pub const TEST_DEFAULT_TOKEN_TIMEOUT: u64 = 60;

    pub fn new_token(kind: TokenKind) -> Token {
        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
        Token::new(kind, ISS, &SUB.to_string(), timeout)
    }

    pub fn new_token_application<'a>() -> TokenApplication<'a, InMemoryCache> {
        TokenApplication {
            cache: Arc::new(InMemoryCache),
            timeout: Duration::from_secs(999),
            token_issuer: "unit_tests",
            private_key: &PRIVATE_KEY,
            public_key: &PUBLIC_KEY,
        }
    }

    #[tokio::test]
    async fn verify_token_should_not_fail() {
        let app = new_token_application();
        let token = new_token(TokenKind::Session);
        let signed = app.sign(&token).unwrap();
        println!(">>>>>>>>>>>>>> {}", signed);

        let claims = app.decode(&signed).unwrap();

        assert!(matches!(&claims.knd, TokenKind::Session));
    }

    #[tokio::test]
    async fn decode_token_expired_should_fail() {
        let mut claim = new_token(TokenKind::Session);
        claim.exp = SystemTime::now() - Duration::from_secs(61);

        let token = crypto::sign_jwt(&PRIVATE_KEY, claim).unwrap();
        let app = new_token_application();

        app.decode(&token)
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn decode_token_invalid_should_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session))
            .unwrap()
            .replace('A', "a");

        let app = new_token_application();
        app.decode(&token)
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }
}
