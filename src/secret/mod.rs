pub mod framework;
pub mod application;
pub mod domain;

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use crate::metadata::domain::InnerMetadata;
    use super::domain::{Secret, SecretRepository};

    struct Mock;

    impl SecretRepository for Mock {
        fn find(&self, _id: &str) -> Result<Secret, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, _secret: &mut Secret) -> Result<(), Box<dyn Error>> {
            Ok(())
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
            //repo: &Mock,
        }
    }
}