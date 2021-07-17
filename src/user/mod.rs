pub mod framework;
pub mod application;
pub mod domain;

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::time::SystemTime;
    use crate::metadata::domain::Metadata;
    use super::domain::{User, UserRepository};

    struct Mock {}
    
    impl UserRepository for &Mock {
        fn find(&self, email: &str) -> Result<User, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
            user.id = 999;
            Ok(())
        }

        fn delete(&self, user: &User) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }

    #[test]
    fn user_new_ok() {
        const EMAIL: &str = "dummy@example.com";
        let mock_impl = &Mock{};

        let before = SystemTime::now();
        let user = User::new(Box::new(mock_impl),
                             Metadata::now(),
                             EMAIL).unwrap();
        
        let after = SystemTime::now();

        assert_eq!(user.id, 999); 
        assert_eq!(user.email, EMAIL);
        assert!(user.secret.is_none());
    }

    #[test]
    fn user_new_ko() {
        const EMAIL: &str = "not_an_email";
        let mock_impl = &Mock{};

        let user = User::new(Box::new(mock_impl),
                             Metadata::now(),
                             EMAIL);
    
        assert!(user.is_err());
    }
}