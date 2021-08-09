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
mod tests {
    use std::error::Error;
    use crate::metadata::tests::new_metadata;
    use super::domain::{User, UserRepository};

    const PWD: &str = "ABCDEF1234567890";

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

    #[test]
    fn user_new_ok() {
        const EMAIL: &str = "dummy@example.com";

        let meta = new_metadata();
        let user = User::new(meta,
                             EMAIL,
                             PWD).unwrap();

        assert_eq!(user.id, 999); 
        assert_eq!(user.email, EMAIL);
        assert!(user.secret.is_none());
    }

    #[test]
    fn user_new_ko() {
        const EMAIL: &str = "not_an_email";

        let meta = new_metadata();
        let user = User::new(meta,
                             EMAIL,
                             PWD);
    
        assert!(user.is_err());
    }
}