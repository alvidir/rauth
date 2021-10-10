use std::error::Error;
use std::time::{SystemTime, Duration};

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;
use crate::time::unix_timestamp;
use crate::security;

pub trait UserRepository {
    fn find(&self, id: i32) -> Result<User, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct User {
    pub(super) id: i32,
    pub(super) email: String, // hash of the email
    pub(super) password: String,
    pub(super) verified_at: Option<SystemTime>,
    pub(super) secret: Option<Secret>,
    pub(super) meta: Metadata,
}

impl User {
    pub fn new(meta: Metadata,
               email: &str,
               password: &str) -> Result<Self, Box<dyn Error>> {
        
        regex::match_regex(regex::EMAIL, email)?;
        regex::match_regex(regex::BASE64, password)?;
        
        let user = User {
            id: 0,
            email: email.to_string(),
            password: security::format_password(password),
            verified_at: None,
            secret: None,
            meta: meta,
        };

        Ok(user)
    }

    /// if the user was not verified before, sets the current time as its verification time
    pub(super) fn verify(&mut self) -> Result<(), Box<dyn Error>> {
        if self.verified_at.is_some() {
            return Err("already verified".into());
        }

        self.verified_at = Some(SystemTime::now());
        self.meta.touch();
        Ok(())
    }

    /// sets the secret and return the old one if any
    pub(super) fn set_secret(&mut self, secret: Option<Secret>) -> Option<Secret> {
        let old_secret = self.secret.clone();
        self.secret = secret;
        
        old_secret
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn get_secret(&self) -> &Option<Secret> {
        &self.secret
    }

    /// if true, the user its verified, else is not
    pub fn is_verified(&self) -> bool {
        self.verified_at.is_some()
    }

    // checks the provided password matches the user's one
    pub fn match_password(&self, password: &str) -> bool {
        security::format_password(password) == self.password
    }
}

// token for email-verification
#[derive(Serialize, Deserialize)]
pub struct Token {
    pub(super) exp: usize,          // expiration time (as UTC timestamp) - required
    pub(super) iat: SystemTime,     // issued at: creation time
    pub(super) iss: String,         // issuer
    pub(super) sub: i32,            // subject: the user id
}

impl Token {
    pub fn new(user: &User, timeout: Duration) -> Self {
        Token {
            exp: unix_timestamp(SystemTime::now() + timeout),
            iat: SystemTime::now(),
            iss: "rauth.alvidir.com".to_string(),
            sub: user.id,
        }
    }
}


#[cfg(test)]
pub mod tests {
    use std::time::{SystemTime, Duration};
    use crate::metadata::domain::tests::new_metadata;
    use crate::time::unix_timestamp;
    use super::{User, Token};
        
    pub fn new_user() -> User {
        User{
            id: 999,
            email: "dummy@test.com".to_string(),
            password: "ABCDEF1234567890".to_string(),
            verified_at: None,
            secret: None,
            meta: new_metadata(),
        }
    }

    pub fn new_user_custom(id: i32, email: &str) -> User {
        User{
            id: id,
            email: email.to_string(),
            password: "ABCDEF1234567890".to_string(),
            verified_at: None,
            secret: None,
            meta: new_metadata(),
        }
    }

    #[test]
    fn user_new_should_not_fail() {
        const PWD: &str = "ABCDEF1234567890";
        const EMAIL: &str = "dummy@test.com";

        let meta = new_metadata();
        let user = User::new(meta,
                             EMAIL,
                             PWD).unwrap();

        assert_eq!(user.id, 0); 
        assert_eq!(user.email, EMAIL);
        assert!(user.secret.is_none());
    }

    #[test]
    fn user_new_wrong_email_should_fail() {
        const PWD: &str = "ABCDEF1234567890";
        const EMAIL: &str = "not_an_email";

        let meta = new_metadata();
        let user = User::new(meta,
                             EMAIL,
                             PWD);
    
        assert!(user.is_err());
    }

    #[test]
    fn user_new_wrong_password_should_fail() {
        const PWD: &str = "ABCDEFG1234567890";
        const EMAIL: &str = "dummy@test.com";

        let meta = new_metadata();
        let user = User::new(meta,
                             EMAIL,
                             PWD);
    
        assert!(user.is_err());
    }

    #[test]
    fn user_verify_should_not_fail() {
        let mut user = new_user();
        assert!(!user.is_verified());

        let before = SystemTime::now();
        assert!(user.verify().is_ok());
        let after = SystemTime::now();

        assert!(user.verified_at.is_some());
        let time = user.verified_at.unwrap();
        assert!(time >= before && time <= after);

        assert!(user.is_verified());
    }

    #[test]
    fn user_verify_should_fail() {
        let mut user = new_user();
        user.verified_at = Some(SystemTime::now());

        assert!(user.verify().is_err());
    }

    #[test]
    fn user_match_password_should_fail() {
        let user = new_user();
        assert!(!user.match_password("TESTER"));
    }

    #[test]
    fn user_token_should_not_fail() {
        let user = new_user();
        let timeout = Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(&user, timeout);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);     
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));       
        assert_eq!("rauth.alvidir.com", claim.iss);
        assert_eq!(user.id, claim.sub);
    }

    #[test]
    #[cfg(feature = "integration-tests")]
    fn user_token_encode_should_not_fail() {
        use crate::security;
        
        dotenv::dotenv().unwrap();

        let user = new_user();
        let timeout = Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(&user, timeout);
        let after = SystemTime::now();
        
        let token = security::encode_jwt(claim).unwrap();
        let claim = security::decode_jwt::<Token>(&token).unwrap();

        assert!(claim.iat >= before && claim.iat <= after);     
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));       
        assert_eq!("rauth.alvidir.com", claim.iss);
        assert_eq!(user.id, claim.sub);
    }

    #[test]
    #[cfg(feature = "integration-tests")]
    fn user_token_expired_should_fail() {
        use std::thread::sleep;
        use crate::security;

        dotenv::dotenv().unwrap();

        let user = new_user();
        let timeout = Duration::from_secs(0);

        let claim = Token::new(&user, timeout);
        let token = security::encode_jwt(claim).unwrap();
        
        sleep(Duration::from_secs(1));
        assert!(security::decode_jwt::<Token>(&token).is_err());
    }
}