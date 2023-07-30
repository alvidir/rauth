use super::domain::{Token, TokenKind};
use crate::cache::Cache;
use crate::crypto;
use crate::result::{Error, Result};
use std::sync::Arc;
use std::time::Duration;

pub struct TokenApplication<'a, C: Cache> {
    pub cache: Arc<C>,
    pub timeout: Duration,
    pub token_issuer: &'a str,
    pub private_key: &'a [u8],
    pub public_key: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    /// Determines if the [Token] to be generated must be persisted or not.
    pub store: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self { store: true }
    }
}

#[derive(Debug, Clone)]
pub struct VerifyOptions {
    pub must_exists: bool,
    pub kind: Option<TokenKind>,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            must_exists: true,
            kind: None,
        }
    }
}

impl VerifyOptions {
    pub fn new(kind: TokenKind) -> Self {
        VerifyOptions {
            kind: Some(kind),
            ..Default::default()
        }
    }
}

impl<'a, T: Cache> TokenApplication<'a, T> {
    #[instrument(skip(self))]
    pub async fn generate(&self, kind: TokenKind, sub: &str) -> Result<Token> {
        Ok(Token::new(kind, self.token_issuer, sub, self.timeout))
    }

    #[instrument(skip(self))]
    pub async fn store(&self, token: &Token) -> Result<String> {
        let signed = crypto::sign_jwt(self.private_key, token)?;

        self.cache
            .save(&token.jti, &signed, Some(self.timeout.as_secs()))
            .await?;

        Ok(signed)
    }

    #[instrument(skip(self))]
    pub async fn decode(&self, token: &str) -> Result<Token> {
        crypto::decode_jwt(self.public_key, token)
    }

    #[instrument(skip(self))]
    pub async fn find(&self, key: &str) -> Result<Token> {
        let token: String = self.cache.find(key).await?;
        let claims = self.decode(&token).await?;
        Ok(claims)
    }

    #[instrument(skip(self))]
    pub async fn verify(&self, token: &Token, options: VerifyOptions) -> Result<()> {
        if let Some(kind) = options.kind {
            if token.knd != kind {
                warn!(
                    token_id = token.jti,
                    token_kind = token.knd.to_string(),
                    expected_kind = kind.to_string(),
                    "checking token's kind",
                );
                return Err(Error::InvalidToken);
            }
        }

        if options.must_exists {
            let present_data: String = self.cache.find(&token.jti).await.map_err(|err| {
                warn!(
                    error = err.to_string(),
                    token_id = token.jti,
                    "finding token by id",
                );
                Error::InvalidToken
            })?;

            let present_token = self.decode(&present_data).await?;
            if token != &present_token {
                error!(token_id = token.jti, "token does not match");
                return Err(Error::InvalidToken);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn revoke(&self, token: &Token) -> Result<()> {
        self.cache.find(&token.jti).await.map_err(|err| {
            warn!(
                error = err.to_string(),
                token_id = token.jti,
                "finding token by id",
            );
            Error::InvalidToken
        })?;

        self.cache.delete(&token.jti).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::TokenApplication;
    use crate::cache::tests::InMemoryCache;
    use crate::time;
    use crate::token::application::VerifyOptions;
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
            token_issuer: "dummy",
            private_key: &PRIVATE_KEY,
            public_key: &PUBLIC_KEY,
        }
    }

    #[tokio::test]
    async fn verify_token_should_not_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session)).unwrap();
        let app = new_token_application();
        let claims = app.decode(&token).await.unwrap();
        app.verify(&claims, VerifyOptions::new(TokenKind::Session))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn decode_token_expired_should_fail() {
        let mut claim = new_token(TokenKind::Session);
        claim.exp = time::unix_timestamp(SystemTime::now() - Duration::from_secs(61));

        let token = crypto::sign_jwt(&PRIVATE_KEY, claim).unwrap();
        let app = new_token_application();

        app.decode(&token)
            .await
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
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_wrong_kind_should_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session)).unwrap();
        let app = new_token_application();
        let claims = app.decode(&token).await.unwrap();
        app.verify(&claims, VerifyOptions::new(TokenKind::Verification))
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_not_present_should_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session)).unwrap();
        let app = new_token_application();
        let claims = app.decode(&token).await.unwrap();
        app.verify(&claims, VerifyOptions::new(TokenKind::Verification))
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_mismatch_should_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session)).unwrap();
        let app = new_token_application();
        let claims = app.decode(&token).await.unwrap();
        app.verify(&claims, VerifyOptions::new(TokenKind::Verification))
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }
}
