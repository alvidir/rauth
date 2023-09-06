use super::{
    domain::{Payload, Token, TokenKind},
    error::{Error, Result},
};
use crate::cache::Cache;
use crate::on_error;
use jsonwebtoken::{Algorithm as JwtAlgorithm, DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;
use std::time::Duration;

pub struct TokenApplication<'a, C: Cache> {
    pub timeout: Duration,
    pub token_issuer: &'a str,
    pub private_key: &'a [u8],
    pub public_key: &'a [u8],
    pub cache: Arc<C>,
}

impl<'a, C> TokenApplication<'a, C>
where
    C: Cache,
{
    /// Returns the resulting token of signing the given payload.
    pub fn encode(&self, payload: Payload) -> Result<Token> {
        let header = Header::new(JwtAlgorithm::ES256);
        let key = EncodingKey::from_ec_pem(self.private_key).map_err(Error::from)?;

        jsonwebtoken::encode(&header, &payload, &key)
            .map(Token::from)
            .map_err(Error::from)
    }

    /// Returns the payload of the given token.
    pub fn decode(&self, token: Token) -> Result<Payload> {
        let validation = Validation::new(JwtAlgorithm::ES256);
        let key = DecodingKey::from_ec_pem(self.public_key)
            .map_err(on_error!(Error, "decoding elliptic curve keypair"))?;

        jsonwebtoken::decode(token.as_ref(), &key, &validation)
            .map(|token| token.claims)
            .map_err(on_error!("checking token's signature"))
    }

    /// Returns a new token payload with the given kind and subject.
    #[instrument(skip(self))]
    pub fn payload(&self, kind: TokenKind, sub: &str) -> Payload {
        Payload::new(kind, self.token_issuer, sub, self.timeout)
    }

    /// Stores the given [Payload] in the cache.
    #[instrument(skip(self))]
    pub async fn store(&self, payload: &Payload) -> Result<()> {
        self.cache
            .save(&payload.jti, payload, Some(payload.timeout()))
            .await
    }

    /// Retrives the [Payload] associated to the given token ID, if any.
    #[instrument(skip(self))]
    pub async fn find(&self, id: &str) -> Result<Payload> {
        self.cache
            .find::<_, Error>(id)
            .await?
            .ok_or(Error::NotFound)
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
    use crate::token::error::Error;
    use base64::{engine::general_purpose, Engine as _};
    use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};
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
        let signed = app.encode(token).unwrap();

        let claims = app.decode(signed).unwrap();

        assert!(matches!(&claims.knd, TokenKind::Session));
    }

    #[tokio::test]
    async fn decode_token_expired_should_fail() {
        let mut claim = new_token(TokenKind::Session);
        claim.exp = SystemTime::now() - Duration::from_secs(61);

        let app = new_token_application();
        let token = app.encode(claim).unwrap();

        app.decode(token.into())
            .map_err(|err| {
                let want = Error::Invalid(JwtError::from(JwtErrorKind::ExpiredSignature));
                assert!(matches!(err, want))
            })
            .unwrap_err();
    }

    // #[tokio::test]
    // async fn decode_token_invalid_should_fail() {
    //     let token = crypto::encode_jwt(&PRIVATE_KEY, new_token(TokenKind::Session))
    //         .unwrap()
    //         .replace('A', "a");

    //     let app = new_token_application();
    //     app.payload_from(token.into())
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[test]
    // fn token_encode_should_not_fail() {
    //     const ISS: &str = "test";
    //     const SUB: i32 = 999;
    //     let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
    //     let claim = Payload::new(TokenKind::Session, ISS, &SUB.to_string(), timeout);

    //     let secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
    //     let token = crypto::encode_jwt::<_, String>(&secret, claim).unwrap();

    //     let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
    //     let _ = crypto::decode_jwt::<Payload, String>(&public, &token).unwrap();
    // }

    // #[test]
    // fn expired_token_verification_should_fail() {
    //     use crate::crypto;

    //     const ISS: &str = "test";
    //     const SUB: i32 = 999;

    //     let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
    //     let mut claim = Payload::new(TokenKind::Session, ISS, &SUB.to_string(), timeout);
    //     claim.exp = SystemTime::now() - Duration::from_secs(61);

    //     let secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
    //     let token = crypto::encode_jwt::<_, String>(&secret, claim).unwrap();
    //     let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();

    //     assert!(crypto::decode_jwt::<Payload, String>(&public, &token).is_err());
    // }
}
