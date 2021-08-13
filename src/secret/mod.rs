pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::MongoSecretRepository = {
        framework::MongoSecretRepository
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::SecretRepository> {
    Box::new(&*REPO_PROVIDER)
}

#[cfg(test)]
pub mod tests {
    use crate::metadata::domain::InnerMetadata;
    use super::domain::Secret;

    pub fn new_secret() -> Secret {
        let inner_meta = InnerMetadata::new();

        Secret {
            id: "".to_string(),
            data: "this is a secret".as_bytes().to_vec(),
            meta: inner_meta,
        }
    }

    #[test]
    fn secret_new() {
        let data = "testing".as_bytes();
        let secret = Secret::new(data);

        assert_eq!("", secret.id); 
        assert_eq!("testing".as_bytes(), secret.data);
    }
}