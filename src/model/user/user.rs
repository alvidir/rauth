use crate::model::user::traits::*;
use crate::model::client::traits::Controller as ClientController;

//use diesel;
//use diesel::prelude::*;
//use diesel::mysql::MysqlConnection;
//
//#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub emails: Vec<String>,
    client: Box<dyn ClientController>,
}

impl User {
    pub fn new(client: Box<dyn ClientController>, email: String) -> Self {
        User{
            id: 0,
            emails: vec!{email},
            client: client,
        }
    }
}

impl Controller for User {
    fn get_addr(&self) -> &str {
        &self.emails[0]
    }

}