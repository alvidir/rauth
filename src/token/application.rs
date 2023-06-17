use super::domain::SignedToken;
use super::domain::{Token, TokenDefinition, TokenKind};
use crate::crypto;
use crate::result::{Error, Result};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

#[async_trait]
pub trait TokenRepository {
    async fn find(&self, key: &str) -> Result<String>;
    async fn save(&self, key: &str, token: &str, expire: Option<u64>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

pub struct TokenApplication<'a, T: TokenRepository> {
    pub token_repo: Arc<T>,
    pub timeout: Duration,
    pub token_issuer: &'a str,
    pub private_key: &'a [u8],
    pub public_key: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct GenerateOptions {
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

impl<'a, T: TokenRepository> TokenApplication<'a, T> {
    #[instrument(skip(self))]
    pub async fn generate(
        &self,
        kind: TokenKind,
        sub: &str,
        secret: Option<&str>,
        options: GenerateOptions,
    ) -> Result<SignedToken> {
        let token = Token::new(self.token_issuer, sub, self.timeout, kind, secret);
        let signed = crypto::sign_jwt(self.private_key, &token)?;

        if options.store {
            self.token_repo
                .save(&token.get_id(), &signed, Some(self.timeout.as_secs()))
                .await?;
        }

        Ok(SignedToken {
            id: token.get_id(),
            signature: signed,
        })
    }

    #[instrument(skip(self))]
    pub async fn decode(&self, token: &str) -> Result<Token> {
        crypto::decode_jwt(self.public_key, token)
    }

    #[instrument(skip(self))]
    pub async fn retrieve(&self, key: &str) -> Result<Token> {
        let token = self.token_repo.find(key).await?;
        let claims = self.decode(&token).await?;
        Ok(claims)
    }

    #[instrument(skip(self))]
    pub async fn verify(&self, token: &Token, options: VerifyOptions) -> Result<()> {
        if let Some(kind) = options.kind {
            if *token.get_kind() != kind {
                warn!(
                    token_id = token.get_id(),
                    token_kind = token.get_kind().to_string(),
                    expected_kind = kind.to_string(),
                    "checking token's kind",
                );
                return Err(Error::InvalidToken);
            }
        }

        if options.must_exists {
            let key = token.get_id();
            let present_data = self.token_repo.find(&key).await.map_err(|err| {
                warn!(
                    error = err.to_string(),
                    token_id = key,
                    "finding token by id",
                );
                Error::InvalidToken
            })?;

            let present_token = self.decode(&present_data).await?;
            if token != &present_token {
                error!(token_id = key, "token does not match");
                return Err(Error::InvalidToken);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn revoke(&self, token: &Token) -> Result<()> {
        let key = token.get_id();
        self.token_repo.find(&key).await.map_err(|err| {
            warn!(
                error = err.to_string(),
                token_id = key,
                "finding token by id",
            );
            Error::InvalidToken
        })?;

        self.token_repo.delete(&key).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::{TokenApplication, TokenRepository};
    use crate::time;
    use crate::token::application::VerifyOptions;
    use crate::token::domain::{Token, TokenKind};
    use crate::{
        crypto,
        result::{Error, Result},
    };
    use async_trait::async_trait;
    use base64::{engine::general_purpose, Engine as _};
    use lazy_static::lazy_static;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    lazy_static! {
        pub static ref PRIVATE_KEY: Vec<u8> = general_purpose::STANDARD.decode(
            b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg=="
        ).unwrap();
        pub static ref PUBLIC_KEY: Vec<u8> = general_purpose::STANDARD.decode(
            b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg=="
        ).unwrap();
    }

    type MockFnFind = Option<fn(this: &TokenRepositoryMock, key: &str) -> Result<String>>;
    type MockFnSave = Option<
        fn(this: &TokenRepositoryMock, key: &str, token: &str, expire: Option<u64>) -> Result<()>,
    >;
    type MockFnDelete = Option<fn(this: &TokenRepositoryMock, key: &str) -> Result<()>>;

    pub const TEST_DEFAULT_TOKEN_TIMEOUT: u64 = 60;

    pub fn new_token(kind: TokenKind) -> Token {
        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
        Token::new(ISS, &SUB.to_string(), timeout, kind, None)
    }

    #[derive(Default, Clone)]
    pub struct TokenRepositoryMock {
        pub fn_find: MockFnFind,
        pub fn_save: MockFnSave,
        pub fn_delete: MockFnDelete,
        pub token: String,
    }

    #[async_trait]
    impl TokenRepository for TokenRepositoryMock {
        async fn find(&self, key: &str) -> Result<String> {
            if let Some(fn_find) = self.fn_find {
                return fn_find(self, key);
            }

            Ok(self.token.clone())
        }

        async fn save(&self, key: &str, token: &str, expire: Option<u64>) -> Result<()> {
            if let Some(fn_save) = self.fn_save {
                return fn_save(self, key, token, expire);
            }

            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<()> {
            if let Some(fn_delete) = self.fn_delete {
                return fn_delete(self, key);
            }

            Ok(())
        }
    }

    pub fn new_token_application<'a, T: TokenRepository + Default>(
        token_repo: Option<T>,
    ) -> TokenApplication<'a, T> {
        TokenApplication {
            token_repo: Arc::new(token_repo.unwrap_or_default()),
            timeout: Duration::from_secs(999),
            token_issuer: "dummy",
            private_key: &PRIVATE_KEY,
            public_key: &PUBLIC_KEY,
        }
    }

    #[tokio::test]
    async fn verify_token_should_not_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session)).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let app = new_token_application(Some(token_repo));
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
        let app = new_token_application::<TokenRepositoryMock>(None);

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
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let app = new_token_application(Some(token_repo));
        app.decode(&token)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_wrong_kind_should_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session)).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let app = new_token_application(Some(token_repo));
        let claims = app.decode(&token).await.unwrap();
        app.verify(&claims, VerifyOptions::new(TokenKind::Verification))
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_not_present_should_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session)).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|_: &TokenRepositoryMock, _: &str| -> Result<String> {
                Err(Error::NotFound)
            }),
            ..Default::default()
        };

        let app = new_token_application(Some(token_repo));
        let claims = app.decode(&token).await.unwrap();
        app.verify(&claims, VerifyOptions::new(TokenKind::Verification))
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_mismatch_should_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session)).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|_: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok("hello world".to_string())
            }),
            ..Default::default()
        };

        let app = new_token_application(Some(token_repo));
        let claims = app.decode(&token).await.unwrap();
        app.verify(&claims, VerifyOptions::new(TokenKind::Verification))
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }
}
