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

#[cfg(test)]
pub mod tests {
    use std::time::SystemTime;
    use crate::metadata::domain::InnerMetadata;
    use crate::app::tests::new_app;
    use crate::session::tests::new_session;
    use super::domain::Directory;

    pub fn new_directory() -> Directory {
        Directory{
            id: "testing".to_string(),
            user: 0,
            app: 0,
            _deadline: SystemTime::now(),
            meta: InnerMetadata::new(),
        }
    }

    #[test]
    fn directory_new() {
        let app = new_app();
        let sess = new_session();

        let dir = Directory::new(&sess, &app);
        
        assert_eq!("", dir.id);
        assert_eq!(dir.app, app.get_id());
        assert_eq!(dir.user, sess.get_user().get_id());
    }
}