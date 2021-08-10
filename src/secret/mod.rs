pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::MongoSecretRepository = {
        framework::MongoSecretRepository
    }; 
}   

#[cfg(not(test))]
pub fn get_repository() -> Box<&'static dyn domain::SecretRepository> {
    Box::new(&*REPO_PROVIDER)
}

#[cfg(test)]
pub fn get_repository() -> Box<dyn domain::SecretRepository> {
    Box::new(tests::Mock)
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use crate::metadata::domain::InnerMetadata;
    use super::domain::{Secret, SecretRepository};

    pub struct Mock;
    impl SecretRepository for Mock {
        fn find(&self, _id: &str) -> Result<Secret, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>> {
            secret.id = "testing".to_string();
            Ok(())
        }

        fn save(&self, _secret: &Secret) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn delete(&self, _secret: &Secret) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }  
    }

    pub fn new_secret() -> Secret {
        let inner_meta = InnerMetadata::new();

        Secret {
            id: "testing".to_string(),
            data: "this is a secret".as_bytes().to_vec(),
            meta: inner_meta,
        }
    }

    #[test]
    fn secret_new() {
        let data = "testing".as_bytes();
        let secret = Secret::new(data).unwrap();

        assert_eq!("testing", secret.id); 
        assert_eq!("testing".as_bytes(), secret.data);
    }
}