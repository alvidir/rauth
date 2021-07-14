use std::error::Error;

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;

pub trait AppRepository {
    fn find(url: &str) -> Result<App, Box<dyn Error>>;
    fn save(app: &mut App) -> Result<(), Box<dyn Error>>;
    fn delete(app: &App) -> Result<(), Box<dyn Error>>;
}

pub struct App {
    pub id: i32,
    pub url: String,
    pub secret: Secret,
    pub meta: Metadata,
}

impl App {
    pub fn new<'a>(url: &'a str, secret: Secret) -> Result<Self, Box<dyn Error>> {
        regex::match_regex(regex::URL, url)?;
        //let repo = super::get_repository()?;

        let app = App {
            id: 0,
            url: url.to_string(),
            secret: secret,
            meta: Metadata::new(),
        };

        
        //repo.save(&mut app)?;
        Ok(app)
    }
}