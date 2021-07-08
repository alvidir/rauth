use std::error::Error;

use crate::regex::*;
use crate::meta::domain::Metadata;

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