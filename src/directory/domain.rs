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
               app: &App) -> Result<Self, Box<dyn Error>> {

        let mut directory = Directory {
            id: "".to_string(),
            user: sess.get_user().get_id(),
            app: app.get_id(),
            _deadline: sess.get_deadline(),
            meta: InnerMetadata::new(),
        };

        super::get_repository().create(&mut directory)?;
        Ok(directory)
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn _set_deadline(&mut self, deadline: SystemTime) {
        self._deadline = deadline;
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        super::get_repository().save(self)
    }

    pub fn _delete(&self) -> Result<(), Box<dyn Error>> {
        super::get_repository().delete(self)
    }
}