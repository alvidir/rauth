use super::{
    domain::{Kind, Payload, Token},
    error::Result,
    Error,
};
use crate::{cache::Cache, on_error};
use jsonwebtoken::{Algorithm as JwtAlgorithm, DecodingKey, EncodingKey, Header, Validation};
use std::{sync::Arc, time::Duration};

pub struct TokenService<'a, C> {
    pub timeout: Duration,
    pub token_issuer: &'a str,
    pub decode: DecodingKey,
    pub encode: EncodingKey,
    pub cache: Arc<C>,
}

impl<'a, C> TokenService<'a, C>
where
    C: Cache,
{
    /// Returns a new payload with the default values.
    #[instrument(skip(self))]
    pub fn new_payload<S: AsRef<str>>(&self, kind: Kind, sub: S) -> Payload {
        Payload::new(kind, self.token_issuer, sub.as_ref(), self.timeout)
    }

    /// Issues a new token containing the given payload.
    #[instrument(skip(self))]
    pub async fn issue(&self, payload: Payload) -> Result<Token> {
        let token = self.encode(&payload)?;
        self.store(&payload).await?;
        Ok(token)
    }

    /// Consumes the token, returning its payload if, and only if, the token is valid and of the expected kind.
    #[instrument(skip(self))]
    pub async fn consume(&self, kind: Kind, token: Token) -> Result<Payload> {
        let payload = self.decode(token)?;

        let Some(actual_payload) = self.find(&payload.jti).await? else {
            return Error::RejectedToken.into();
        };

        if payload != actual_payload {
            return Error::Collision.into();
        }

        if payload.kind() != kind {
            return Error::WrongToken.into();
        }

        Ok(payload)
    }

    /// Invalidates the token with the given ID, if any.
    #[instrument(skip(self))]
    pub async fn revoke(&self, payload: &Payload) -> Result<()> {
        self.cache.delete(&payload.jti).await.map_err(Into::into)
    }

    /// Returns the resulting token of signing and encoding the given payload.
    #[instrument(skip(self))]
    fn encode(&self, payload: &Payload) -> Result<Token> {
        let header = Header::new(JwtAlgorithm::ES256);

        jsonwebtoken::encode(&header, payload, &self.encode)
            .map_err(on_error!("encoding payload into a token"))
            .and_then(Token::try_from)
    }

    /// Returns the payload of the given token.
    #[instrument(skip(self))]
    fn decode(&self, token: Token) -> Result<Payload> {
        // TODO: consider moving validation into a once_cell
        let mut validation = Validation::new(JwtAlgorithm::ES256);
        validation.set_issuer(&[self.token_issuer]);

        jsonwebtoken::decode(token.as_ref(), &self.decode, &validation)
            .map(|token| token.claims)
            .map_err(on_error!("decoding payload from token"))
    }

    /// Stores the given [Payload] in the cache.
    #[instrument(skip(self))]
    async fn store(&self, payload: &Payload) -> Result<()> {
        self.cache
            .save(&payload.jti, payload, Some(payload.timeout()))
            .await
            .map_err(Into::into)
    }

    /// Retrives the [Payload] associated to the given token ID, if any.
    #[instrument(skip(self))]
    async fn find(&self, id: &str) -> Result<Option<Payload>> {
        self.cache.find(id).await.map_err(Into::into)
    }
}

#[cfg(test)]
pub mod tests {
    use super::TokenService;
    use crate::cache::tests::InMemoryCache;
    use crate::token::domain::{Kind, Payload, Token};
    use crate::token::error::Error;
    use base64::{engine::general_purpose, Engine as _};
    use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};
    use jsonwebtoken::{DecodingKey, EncodingKey};
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

    pub const TEST_TOKEN_TIMEOUT: u64 = 60;
    pub const TEST_TOKEN_ISSUER: &str = "test";
    pub const TEST_TOKEN_SUBJECT: &str = "999";

    pub fn new_payload(kind: Kind) -> Payload {
        let timeout = Duration::from_secs(TEST_TOKEN_TIMEOUT);
        Payload::new(
            kind,
            TEST_TOKEN_ISSUER,
            &TEST_TOKEN_SUBJECT.to_string(),
            timeout,
        )
    }

    pub fn new_token_application<'a>() -> TokenService<'a, InMemoryCache> {
        TokenService {
            timeout: Duration::from_secs(999),
            token_issuer: "test",
            decode: DecodingKey::from_ec_pem(&PUBLIC_KEY).unwrap(),
            encode: EncodingKey::from_ec_pem(&PRIVATE_KEY).unwrap(),
            cache: Arc::new(InMemoryCache),
        }
    }

    #[tokio::test]
    async fn consume_token_should_not_fail() {
        let app = new_token_application();
        let payload = app.new_payload(Kind::Session, TEST_TOKEN_SUBJECT);
        let token = app.issue(payload).await.unwrap();

        let claims = app.consume(Kind::Session, token).await.unwrap();
        assert!(matches!(&claims.knd, Kind::Session));
    }

    #[tokio::test]
    async fn consume_expired_token_should_fail() {
        let app = new_token_application();
        let mut payload = app.new_payload(Kind::Session, TEST_TOKEN_SUBJECT);
        payload.exp = SystemTime::now() - Duration::from_secs(61);

        let token = app.encode(&payload).unwrap();

        app.consume(Kind::Session, token)
            .await
            .map_err(|err| {
                let want: Error = JwtError::from(JwtErrorKind::ExpiredSignature).into();
                assert!(matches!(err, want))
            })
            .unwrap_err();
    }

    #[tokio::test]
    async fn consume_corrupt_token_should_fail() {
        let app = new_token_application();
        let payload = app.new_payload(Kind::Session, TEST_TOKEN_SUBJECT);
        let token = app.issue(payload).await.unwrap();
        let corrupted: Token = token.as_ref().replace('A', "a").try_into().unwrap();

        let err = app.consume(Kind::Session, corrupted).await.unwrap_err();
        assert!(matches!(err, Error::Jwt(_)));
    }

    #[tokio::test]
    async fn consume_wrong_token_kind_should_fail() {
        let app = new_token_application();
        let payload = app.new_payload(Kind::Session, TEST_TOKEN_SUBJECT);
        let token = app.issue(payload).await.unwrap();

        let err = app.consume(Kind::Verification, token).await.unwrap_err();
        assert!(matches!(err, Error::Jwt(_)));
    }
}
