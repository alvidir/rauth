use std::error::Error;
use crate::user::domain::User;
use crate::app::domain::App;
use crate::metadata::domain::Metadata;


pub trait DirectoryRepository {
    fn find(&self, id: &str) -> Result<Directory, Box<dyn Error>>;
    fn save(&self, secret: &mut Directory) -> Result<(), Box<dyn Error>>;
    fn delete(&self, secret: &Directory) -> Result<(), Box<dyn Error>>;
}

pub struct Directory {
    pub id: String,
    pub user: i32,
    pub app: i32,
    pub meta: Metadata,
}

impl Directory {
    pub fn new(dir_repo: Box<dyn DirectoryRepository>,
               user: &User,
               app: &App) -> Result<Self, Box<dyn Error>> {

        let mut directory = Directory {
            id: "".to_string(),
            user: user.id,
            app: app.id,
            meta: Metadata::now(),
        };

        dir_repo.save(&mut directory)?;
        Ok(directory)
    }
}