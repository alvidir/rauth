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
    pub id: i32,
    pub url: String,
    pub secret: Secret,
    pub meta: Metadata,
}

impl App {
    pub fn new<'a>(app_repo: &dyn AppRepository,
                   secret: Secret,
                   meta: Metadata,
                   url: &'a str) -> Result<Self, Box<dyn Error>> {
        
        regex::match_regex(regex::URL, url)?;

        let mut app = App {
            id: 0,
            url: url.to_string(),
            secret: secret,
            meta: meta,
        };
        
        app_repo.save(&mut app)?;
        Ok(app)
    }
}