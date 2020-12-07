use crate::model::app::traits::Controller;
use crate::model::client::traits::Controller as ClientController;

//use diesel;
//use diesel::prelude::*;
//use diesel::mysql::MysqlConnection;
//
//#[derive(Queryable)]
pub struct App {
    pub id: i32,
    pub description: String,
    pub url: String,
    client: Box<dyn ClientController>,
}

impl App {
    pub fn new(client: Box<dyn ClientController>, url: String) -> Self {
        App{
            id: 0,
            description: "".to_string(),
            url: url,
            client: client,
        }
    }
}

impl Controller for App {
    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_addr(&self) -> &str {
        &self.url
    }
}