use std::error::Error;
use crate::{regex, constants};
use crate::metadata::domain::Metadata;

pub struct User {
    pub(super) id: i32,
    pub(super) name: String,
    pub(super) email: String,
    pub(super) password: String,
    pub(super) meta: Metadata,
}

impl User {
    pub fn new(email: &str,
               password: &str) -> Result<Self, Box<dyn Error>> {
        
        regex::match_regex(regex::EMAIL, email)
            .map_err(|err| {
                info!("{} validating email's format: {}", constants::ERR_INVALID_FORMAT, err);
                constants::ERR_INVALID_FORMAT
            })?;

        regex::match_regex(regex::BASE64, password)
            .map_err(|err| {
                info!("{} validating password's format: {}", constants::ERR_INVALID_FORMAT, err);
                constants::ERR_INVALID_FORMAT
            })?;
        
        let user = User {
            id: 0,
            name: email.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            meta: Metadata::new(),
        };

        Ok(user)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn match_password(&self, password: &str) -> bool {
        password == self.password
    }

    pub fn set_password(&mut self, password: &str) -> Result<(), Box<dyn Error>> {
        regex::match_regex(regex::BASE64, password)
            .map_err(|err| {
                info!("{} validating password's format: {}", constants::ERR_INVALID_FORMAT, err);
                constants::ERR_INVALID_FORMAT
            })?;
            
        self.password = password.to_string();
        Ok(())
    }
}


#[cfg(test)]
pub mod tests {
    use crate::constants;
    use crate::metadata::domain::tests::new_metadata;
    use super::User;

    pub const TEST_DEFAULT_USER_ID: i32 = 999;
    pub const TEST_DEFAULT_USER_NAME: &str = "dummyuser";
    pub const TEST_DEFAULT_USER_EMAIL: &str = "dummy@test.com";
    pub const TEST_DEFAULT_USER_PASSWORD: &str = "ABCDEF1234567890";
        
    pub fn new_user() -> User {
        User{
            id: TEST_DEFAULT_USER_ID,
            name: TEST_DEFAULT_USER_NAME.to_string(),
            email: TEST_DEFAULT_USER_EMAIL.to_string(),
            password: TEST_DEFAULT_USER_PASSWORD.to_string(),
            meta: new_metadata(),
        }
    }

    pub fn new_user_custom(id: i32, email: &str) -> User {
        User{
            id: id,
            name: "customuser".to_string(),
            email: email.to_string(),
            password: TEST_DEFAULT_USER_PASSWORD.to_string(),
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
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_FORMAT));
        
        assert!(result.is_err());
    }

    #[test]
    fn user_new_wrong_password_should_fail() {
        const PWD: &str = "ABCDEFG1234567890";

        let result = User::new(TEST_DEFAULT_USER_EMAIL, PWD)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_FORMAT));
    
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
        assert_eq!(user.match_password("wrong password"), false);
    }
}