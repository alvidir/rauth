pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresUserRepository = {
        framework::PostgresUserRepository
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::UserRepository> {
    return Box::new(&*REPO_PROVIDER);
}