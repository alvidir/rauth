use std::error::Error;

use crate::schema::apps;
use crate::regex::*;

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(Clone)]
#[table_name = "apps"]
pub struct App {
    pub id: i32,
    pub label: String,
    pub url: String,
    pub secret_id: String,
    pub meta_id: i32,
}

impl App {
    pub fn new<'a>(url: &'a str, secret_id: &'a str) -> Result<Self, Box<dyn Error>> {
        match_url(url)?;

        let app = App {
            id: 0,
            label: "".to_string(),
            url: url.to_string(),
            secret_id: secret_id.to_string(),
            meta_id: 0,
        };

        Ok(app)
    }
}