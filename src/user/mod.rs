pub mod framework;
pub mod application;
pub mod domain;

#[cfg(test)]
mod tests {
    use std::error::Error;
    use super::domain::{User, UserRepository};
    use crate::metadata::domain::{Metadata, MetadataRepository};

    struct MockImplementation {}
    
    impl UserRepository for &MockImplementation {
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
    
    impl MetadataRepository for &MockImplementation {
        fn find(&self, id: i32) -> Result<Metadata, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
            meta.id = 999;
            Ok(())
        }

        fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }  
    }

    #[test]
    fn client_new_ok() {
        const EMAIL: &str = "client_new_ok@testing.com";
        let mock_impl = &MockImplementation{};

        let user = User::new(Box::new(mock_impl),
                             Box::new(mock_impl),
                             EMAIL).unwrap();

        assert_eq!(user.id, 999);
        assert_eq!(user.email, EMAIL);
    }
}