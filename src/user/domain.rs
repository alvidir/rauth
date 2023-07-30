use crate::metadata::domain::Metadata;
use crate::{
    email, regex,
    result::{Error, Result},
};

///
#[derive(Default)]
pub(super) struct UserBuilder<'a> {
    email: Option<&'a str>,
    password: Option<&'a str>,
}

impl<'a> UserBuilder<'a> {
    pub fn with_email(mut self, email: &'a str) -> Result<Self> {
        regex::match_regex(regex::EMAIL, email).map_err(|err| {
            warn!(error = err.to_string(), "validating email format",);
            Error::InvalidFormat
        })?;

        self.email = Some(email);
        Ok(self)
    }

    pub fn with_password(mut self, password: &'a str) -> Result<Self> {
        if password.is_empty() {
            return Ok(self);
        }

        regex::match_regex(regex::BASE64, password).map_err(|err| {
            warn!(error = err.to_string(), "validating password format",);
            Error::InvalidFormat
        })?;

        self.password = Some(password);
        Ok(self)
    }

    pub fn build(self) -> Result<User> {
        User::new(
            self.email.unwrap_or_default(),
            self.password.unwrap_or_default(),
        )
    }
}

/// Represents a signed up user
#[derive(Debug)]
pub struct User {
    pub(super) id: i32,
    pub(super) name: String,
    pub(super) email: String,
    pub(super) actual_email: String,
    pub(super) password: String,
    pub(super) meta: Metadata,
}

impl User {
    pub fn new(email: &str, password: &str) -> Result<Self> {
        regex::match_regex(regex::EMAIL, email).map_err(|err| {
            warn!(error = err.to_string(), "validating email format",);
            Error::InvalidFormat
        })?;

        regex::match_regex(regex::BASE64, password).map_err(|err| {
            warn!(error = err.to_string(), "validating password format",);
            Error::InvalidFormat
        })?;

        let user = User {
            id: 0,
            name: email.to_string(),
            email: email.to_string(),
            actual_email: email::actual_email(email),
            password: password.to_string(),
            meta: Metadata::default(),
        };

        Ok(user)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn match_password(&self, password: &str) -> bool {
        password == self.password
    }

    pub fn set_password(&mut self, password: &str) -> Result<()> {
        regex::match_regex(regex::BASE64, password).map_err(|err| {
            info!(error = err.to_string(), "validating password's format",);
            Error::InvalidFormat
        })?;

        self.password = password.to_string();
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::User;
    use crate::metadata::domain::tests::new_metadata;
    use crate::result::Error;
    use crate::{crypto, email};

    pub const TEST_DEFAULT_USER_ID: i32 = 999;
    pub const TEST_DEFAULT_USER_NAME: &str = "dummyuser";
    pub const TEST_DEFAULT_USER_EMAIL: &str = "dummy@test.com";
    pub const TEST_DEFAULT_USER_PASSWORD: &str = "ABCDEF1234567890";
    pub const TEST_DEFAULT_PWD_SUFIX: &str = "sufix";

    pub fn new_user() -> User {
        User {
            id: TEST_DEFAULT_USER_ID,
            name: TEST_DEFAULT_USER_NAME.to_string(),
            email: TEST_DEFAULT_USER_EMAIL.to_string(),
            actual_email: email::actual_email(TEST_DEFAULT_USER_EMAIL),
            password: TEST_DEFAULT_USER_PASSWORD.to_string(),
            meta: new_metadata(),
        }
    }

    pub fn new_user_custom(id: i32, email: &str) -> User {
        User {
            id,
            name: "custom_user".to_string(),
            email: email.to_string(),
            actual_email: email::actual_email(email),
            password: crypto::obfuscate(TEST_DEFAULT_USER_PASSWORD, TEST_DEFAULT_PWD_SUFIX),
            meta: new_metadata(),
        }
    }

    #[test]
    fn user_new_should_not_fail() {
        let user = User::new(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD).unwrap();

        assert_eq!(user.id, 0);
        assert_eq!(user.name, TEST_DEFAULT_USER_EMAIL);
        assert_eq!(user.email, TEST_DEFAULT_USER_EMAIL);
        assert_eq!(user.password, TEST_DEFAULT_USER_PASSWORD);
    }

    #[test]
    fn user_new_wrong_email_should_fail() {
        const EMAIL: &str = "not_an_email";

        let result = User::new(EMAIL, TEST_DEFAULT_USER_PASSWORD)
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidFormat.to_string()));

        assert!(result.is_err());
    }

    #[test]
    fn user_new_wrong_password_should_fail() {
        const PWD: &str = "ABCDEFG1234567890";

        let result = User::new(TEST_DEFAULT_USER_EMAIL, PWD)
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidFormat.to_string()));

        assert!(result.is_err());
    }

    #[test]
    fn user_match_password_should_not_fail() {
        let user = new_user();
        assert!(user.match_password(TEST_DEFAULT_USER_PASSWORD));
    }

    #[test]
    fn user_match_password_should_fail() {
        let user = new_user();
        assert!(!user.match_password("wrong password"));
    }
}
