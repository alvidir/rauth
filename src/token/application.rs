use super::domain::{Payload, Token, TokenKind};
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
    pub fn new_payload(&self, kind: TokenKind, sub: &str) -> Result<Payload> {
        Ok(Payload::new(kind, self.token_issuer, sub, self.timeout))
    }

    /// Returns the resulting [Token] of signing the given [Payload].
    #[instrument(skip(self))]
    pub fn sign(&self, payload: Payload) -> Result<Token> {
        payload.into_token(self.private_key)
    }

    /// Stores the given [Payload] in the cache.
    #[instrument(skip(self))]
    pub async fn store(&self, payload: &Payload) -> Result<()> {
        self.cache
            .save(&payload.jti, payload, Some(payload.timeout()))
            .await
    }

    /// Returns the [Payload] of the given [Token].
    #[instrument(skip(self))]
    pub fn payload(&self, token: Token) -> Result<Payload> {
        crypto::decode_jwt(self.public_key, token.as_ref())
    }

    /// Retrives the [Payload] associated to the given token ID, if any.
    #[instrument(skip(self))]
    pub async fn find(&self, id: &str) -> Result<Payload> {
        self.cache.find(id).await
    }

    /// Removes the entry in the cache with the given token ID.
    #[instrument(skip(self))]
    pub async fn remove(&self, id: &str) -> Result<()> {
        self.cache.delete(id).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::TokenApplication;
    use crate::cache::tests::InMemoryCache;
    use crate::token::domain::{Payload, TokenKind};
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

    pub fn new_token(kind: TokenKind) -> Payload {
        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
        Payload::new(kind, ISS, &SUB.to_string(), timeout)
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
        let signed = app.sign(token).unwrap();

        let claims = app.payload(signed).unwrap();

        assert!(matches!(&claims.knd, TokenKind::Session));
    }

    #[tokio::test]
    async fn decode_token_expired_should_fail() {
        let mut claim = new_token(TokenKind::Session);
        claim.exp = SystemTime::now() - Duration::from_secs(61);

        let token = crypto::encode_jwt(&PRIVATE_KEY, claim).unwrap();
        let app = new_token_application();

        app.payload(token.into())
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn decode_token_invalid_should_fail() {
        let token = crypto::encode_jwt(&PRIVATE_KEY, new_token(TokenKind::Session))
            .unwrap()
            .replace('A', "a");

        let app = new_token_application();
        app.payload(token.into())
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }
}
