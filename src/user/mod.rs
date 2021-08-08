pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresUserRepository = {
        framework::PostgresUserRepository
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::UserRepository> {
    #[cfg(test)]
    return Box::new(&*tests::REPO_TEST);

    #[cfg(not(test))]
    return Box::new(&*REPO_PROVIDER);
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use crate::metadata::tests::new_metadata;
    use super::domain::{User, UserRepository};

    const PWD: &str = "ABCD1234";

    pub struct Mock;
    lazy_static! {
        pub static ref REPO_TEST: Mock = Mock;
    } 
    
    impl UserRepository for Mock {
        fn find(&self, _email: &str) -> Result<User, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
            user.id = 999;
            Ok(())
        }

        fn delete(&self, _user: &User) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }

    #[test]
    fn domain_user_new_ok() {
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
    fn domain_user_new_ko() {
        const EMAIL: &str = "not_an_email";

        let meta = new_metadata();
        let user = User::new(meta,
                             EMAIL,
                             PWD);
    
        assert!(user.is_err());
    }
}