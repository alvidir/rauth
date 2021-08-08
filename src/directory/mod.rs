pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::MongoDirectoryRepository = {
        framework::MongoDirectoryRepository
    }; 
}   

#[cfg(not(test))]
pub fn get_repository() -> Box<&'static dyn domain::DirectoryRepository> {
    Box::new(&*REPO_PROVIDER)
}

#[cfg(test)]
pub fn get_repository() -> Box<dyn domain::DirectoryRepository> {
    Box::new(tests::Mock)
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use crate::app::domain::App;
    use crate::user::domain::User;
    use super::domain::{Directory, DirectoryRepository};

    pub struct Mock;    
    impl DirectoryRepository for Mock {
        fn find(&self, _id: &str) -> Result<Directory, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn find_by_user_and_app(&self, _user_id: i32, _app_id: i32) -> Result<Directory, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn create(&self, secret: &mut Directory) -> Result<(), Box<dyn Error>> {
            secret.id = "999".to_string();
            Ok(())
        }

        fn save(&self, _secret: &Directory) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn delete(&self, _secret: &Directory) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn delete_all_by_app(&self, _app: &App) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn delete_all_by_user(&self, _user: &User) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

    }
}