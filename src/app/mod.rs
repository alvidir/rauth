pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresAppRepository = {
        framework::PostgresAppRepository
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::AppRepository> {
    Box::new(&*REPO_PROVIDER)
}