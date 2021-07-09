use std::error::Error;

use crate::regex::*;
use crate::metadata::domain::Metadata;


pub trait AppRepository {
    fn find(url: &str) -> Result<App, Box<dyn Error>>;
    fn save(app: &mut App) -> Result<(), Box<dyn Error>>;
    fn delete(app: &App) -> Result<(), Box<dyn Error>>;
}

pub struct App {
    pub id: i32,
    pub label: String,
    pub url: String,
    pub meta: Metadata,
}

impl App {
    pub fn new(id: i32, label: &str, url: &str, meta: Metadata) -> Result<Self, Box<dyn Error>> {
        match_url(url)?;

        let app = App {
            id: id,
            label: label.to_string(),
            url: url.to_string(),
            meta: meta,
        };

        Ok(app)
    }
}