pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresUserRepository = {
        framework::PostgresUserRepository
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::UserRepository> {
    return Box::new(&*REPO_PROVIDER);
}

#[cfg(test)]
pub mod tests {
    use std::time::{SystemTime, Duration};
    use std::thread::sleep;
    use crate::metadata::tests::new_metadata;
    use crate::time::unix_timestamp;
    use crate::security;
    use super::domain::{User, Token};
        
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

    #[test]
    fn user_new_ok() {
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
    fn user_email_ko() {
        const PWD: &str = "ABCDEF1234567890";
        const EMAIL: &str = "not_an_email";

        let meta = new_metadata();
        let user = User::new(meta,
                             EMAIL,
                             PWD);
    
        assert!(user.is_err());
    }

    #[test]
    fn user_password_ko() {
        const PWD: &str = "ABCDEFG1234567890";
        const EMAIL: &str = "dummy@test.com";

        let meta = new_metadata();
        let user = User::new(meta,
                             EMAIL,
                             PWD);
    
        assert!(user.is_err());
    }

    #[test]
    fn user_verify_ok() {
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
    fn user_verify_ko() {
        let mut user = new_user();
        user.verified_at = Some(SystemTime::now());

        assert!(user.verify().is_err());
    }

    #[test]
    fn user_match_password_ok() {
        let user = new_user();
        assert!(user.match_password("ABCDEF1234567890"));
    }

    #[test]
    fn user_match_password_ko() {
        let user = new_user();
        assert!(!user.match_password("ABCDEFG1234567890"));
    }

    #[test]
    fn user_token_ok() {
        let user = new_user();
        let timeout = Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(&user, timeout);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);     
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));       
        assert_eq!("oauth.alvidir.com", claim.iss);
        assert_eq!(user.id, claim.sub);
    }

    #[test]
    #[ignore]
    fn user_token_encode() {
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
        assert_eq!("oauth.alvidir.com", claim.iss);
        assert_eq!(user.id, claim.sub);
    }

    #[test]
    #[ignore]
    fn user_token_ko() {
        dotenv::dotenv().unwrap();

        let user = new_user();
        let timeout = Duration::from_secs(0);

        let claim = Token::new(&user, timeout);
        let token = security::encode_jwt(claim).unwrap();
        
        sleep(Duration::from_secs(1));
        assert!(security::decode_jwt::<Token>(&token).is_err());
    }
}