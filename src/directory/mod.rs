pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::MongoDirectoryRepository = {
        framework::MongoDirectoryRepository
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::DirectoryRepository> {
    Box::new(&*REPO_PROVIDER)
}