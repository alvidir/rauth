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