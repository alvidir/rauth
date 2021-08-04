use std::error::Error;

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;

pub trait AppRepository {
    fn find(&self, url: &str) -> Result<App, Box<dyn Error>>;
    fn save(&self, app: &mut App) -> Result<(), Box<dyn Error>>;
    fn delete(&self, app: &App) -> Result<(), Box<dyn Error>>;
}

pub struct App<'a> {
    pub id: i32,
    pub url: &'a str,
    pub secret: Secret<'a>,
    pub meta: Metadata<'a>,

    repo: &'a dyn AppRepository
}

impl<'a> App<'a> {
    pub fn new(app_repo: &'a dyn AppRepository,
               secret: Secret<'a>,
               meta: Metadata<'a>,
               url: &'a str) -> Result<Self, Box<dyn Error>> {
        
        regex::match_regex(regex::URL, url)?;

        let mut app = App {
            id: 0,
            url: url,
            secret: secret,
            meta: meta,

            repo: app_repo,
        };
        
        app_repo.save(&mut app)?;
        Ok(app)
    }

    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        self.repo.save(self)
    }

    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        self.repo.delete(self)
    }
}