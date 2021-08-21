use std::error::Error;
use std::time::SystemTime;
use crate::session::domain::Session;
use crate::app::domain::App;
use crate::user::domain::User;
use crate::metadata::domain::InnerMetadata;

pub trait DirectoryRepository {
    fn find(&self, id: &str) -> Result<Directory, Box<dyn Error>>;
    fn find_by_user_and_app(&self, user_id: i32, app_id: i32) -> Result<Directory, Box<dyn Error>>;
    fn create(&self, secret: &mut Directory) -> Result<(), Box<dyn Error>>;
    fn save(&self, secret: &Directory) -> Result<(), Box<dyn Error>>;
    fn delete(&self, secret: &Directory) -> Result<(), Box<dyn Error>>;
    fn delete_all_by_app(&self, app: &App) -> Result<(), Box<dyn Error>>;
    fn delete_all_by_user(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct Directory {
    pub(super) id: String,
    pub(super) user: i32,
    pub(super) app: i32,
    pub(super) _deadline: SystemTime,
    pub(super) meta: InnerMetadata,
}

impl Directory {
    pub fn new(sess: &Session,
               app: &App) -> Self {

        Directory {
            id: "".to_string(),
            user: sess.get_user().get_id(),
            app: app.get_id(),
            _deadline: sess.get_deadline(),
            meta: InnerMetadata::new(),
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_app(&self) -> i32 {
        self.app
    }

    pub fn _set_deadline(&mut self, deadline: SystemTime) {
        self._deadline = deadline;
    }
}


#[cfg(test)]
pub mod tests {
    use std::time::SystemTime;
    use crate::metadata::domain::InnerMetadata;
    use crate::app::domain::tests::new_app;
    use crate::session::domain::tests::new_session;
    use super::Directory;

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
    fn directory_new_should_not_fail() {
        let app = new_app();
        let sess = new_session();

        let dir = Directory::new(&sess, &app);
        
        assert_eq!("", dir.id);
        assert_eq!(dir.app, app.get_id());
        assert_eq!(dir.user, sess.get_user().get_id());
    }
}