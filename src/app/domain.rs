use std::error::Error;

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;

pub trait AppRepository {
    fn find(&self, id: i32) -> Result<App, Box<dyn Error>>;
    fn find_by_url(&self, url: &str) -> Result<App, Box<dyn Error>>;
    fn create(&self, app: &mut App) -> Result<(), Box<dyn Error>>;
    fn save(&self, app: &App) -> Result<(), Box<dyn Error>>;
    fn delete(&self, app: &App) -> Result<(), Box<dyn Error>>;
}

pub struct App {
    pub(super) id: i32,
    pub(super) url: String,
    pub(super) secret: Secret,
    pub(super) meta: Metadata,
}

impl App {
    pub fn new(secret: Secret,
               meta: Metadata,
               url: &str) -> Result<Self, Box<dyn Error>> {
        
        regex::match_regex(regex::URL, url)?;

        let mut app = App {
            id: 0,
            url: url.to_string(),
            secret: secret,
            meta: meta,
        };
        
        super::get_repository().create(&mut app)?;
        Ok(app)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    /// updates the app into the repository
    pub fn _save(&self) -> Result<(), Box<dyn Error>> {
        super::get_repository().save(self)?;
        self.meta.save()?;
        Ok(())
    }

    /// deletes the application and all its data from the repositories
    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        self.secret.delete()?;
        super::get_repository().delete(self)?;
        self.meta.delete()?;
        Ok(())
    }
}