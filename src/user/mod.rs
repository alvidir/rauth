pub mod framework;
pub mod application;
pub mod domain;

#[cfg(test)]
mod tests {
    use std::error::Error;
    use crate::metadata::domain::{Metadata, MetadataRepository};
    use super::domain::{User, UserRepository};

    const PWD: &str = "ABCD1234";

    struct Mock;
    
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

    impl MetadataRepository for Mock {
        fn find(&self, _id: i32) -> Result<Metadata, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
            meta.id = 999;
            Ok(())
        }

        fn delete(&self, _meta: &Metadata) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }  
    }

    #[test]
    fn domain_user_new_ok() {
        const EMAIL: &str = "dummy@example.com";

        let meta = Metadata::new(&Mock).unwrap();
        let user = User::new(&Mock,
                             meta,
                             EMAIL,
                             PWD).unwrap();

        assert_eq!(user.id, 999); 
        assert_eq!(user.email, EMAIL);
        assert!(user.secret.is_none());
    }

    #[test]
    fn domain_user_new_ko() {
        const EMAIL: &str = "not_an_email";

        let meta = Metadata::new(&Mock).unwrap();
        let user = User::new(&Mock,
                             meta,
                             EMAIL,
                             PWD);
    
        assert!(user.is_err());
    }
}