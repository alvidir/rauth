pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresUserRepository = {
        framework::PostgresUserRepository
    }; 
}   

#[cfg(not(test))]
pub fn get_repository() -> Box<&'static dyn domain::UserRepository> {
    return Box::new(&*REPO_PROVIDER);
}

#[cfg(test)]
pub fn get_repository() -> Box<dyn domain::UserRepository> {
    Box::new(tests::Mock)
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::time::SystemTime;
    use crate::metadata::tests::new_metadata;
    use super::domain::{User, UserRepository};

    pub struct Mock;    
    impl UserRepository for Mock {
        fn find(&self, _id: i32) -> Result<User, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn find_by_email(&self, _email: &str) -> Result<User, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
            user.id = 999;
            Ok(())
        }

        fn save(&self, _user: &User) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn delete(&self, _user: &User) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }
        
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

        assert_eq!(user.id, 999); 
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
}