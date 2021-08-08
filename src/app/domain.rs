use std::error::Error;

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;

pub trait AppRepository {
    fn find(&self, url: &str) -> Result<App, Box<dyn Error>>;
    fn save(&self, app: &mut App) -> Result<(), Box<dyn Error>>;
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
        
        app.save()?;
        Ok(app)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        super::get_repository().save(self)?;
        self.meta.save()?;
        Ok(())
    }

    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        self.secret.delete()?;
        super::get_repository().delete(self)?;
        self.meta.delete()?;
        Ok(())
    }
}