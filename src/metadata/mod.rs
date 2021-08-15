pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresMetadataRepository = {
        framework::PostgresMetadataRepository
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::MetadataRepository> {
    Box::new(&*REPO_PROVIDER)
}