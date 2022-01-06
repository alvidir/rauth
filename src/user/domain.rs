use std::error::Error;
use crate::regex;
use crate::metadata::domain::Metadata;

pub trait UserRepository {
    fn find(&self, id: i32) -> Result<User, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct User {
    pub(super) id: i32,
    pub(super) name: String,
    pub(super) email: String,
    pub(super) password: String,
    pub(super) meta: Metadata,
}

impl User {
    pub fn new(meta: Metadata,
               name: &str,
               email: &str,
               password: &str) -> Result<Self, Box<dyn Error>> {
        
        regex::match_regex(regex::EMAIL, email)?;
        regex::match_regex(regex::BASE64, password)?;
        
        let user = User {
            id: 0,
            name: name.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            meta: meta,
        };

        Ok(user)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    // checks the provided password matches the user's one
    pub fn match_password(&self, password: &str) -> bool {
        password == self.password
    }
}


#[cfg(test)]
pub mod tests {
    use crate::metadata::domain::tests::new_metadata;
    use super::User;
        
    pub fn new_user() -> User {
        User{
            id: 999,
            name: "dummyuser".to_string(),
            email: "dummy@test.com".to_string(),
            password: "ABCDEF1234567890".to_string(),
            meta: new_metadata(),
        }
    }

    pub fn new_user_custom(id: i32, email: &str) -> User {
        User{
            id: id,
            name: "custom user".to_string(),
            email: email.to_string(),
            password: "ABCDEF1234567890".to_string(),
            meta: new_metadata(),
        }
    }

    #[test]
    fn user_new_should_not_fail() {
        const PWD: &str = "ABCDEF1234567890";
        const NAME: &str = "dummy user";
        const EMAIL: &str = "dummy@test.com";

        let meta = new_metadata();
        let user = User::new(meta, NAME, EMAIL, PWD).unwrap();

        assert_eq!(user.id, 0); 
        assert_eq!(user.name, NAME);
        assert_eq!(user.email, EMAIL);
    }

    #[test]
    fn user_new_wrong_email_should_fail() {
        const PWD: &str = "ABCDEF1234567890";
        const NAME: &str = "dummy user";
        const EMAIL: &str = "not_an_email";

        let meta = new_metadata();
        let user = User::new(meta, NAME, EMAIL, PWD);
    
        assert!(user.is_err());
    }

    #[test]
    fn user_new_wrong_password_should_fail() {
        const PWD: &str = "ABCDEFG1234567890";
        const NAME: &str = "dummy user";
        const EMAIL: &str = "dummy@test.com";

        let meta = new_metadata();
        let user = User::new(meta, NAME, EMAIL, PWD);
    
        assert!(user.is_err());
    }

    #[test]
    fn user_match_password_should_fail() {
        let user = new_user();
        assert!(!user.match_password("TESTER"));
    }
}