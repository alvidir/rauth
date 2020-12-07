use crate::model::client::traits::*;
use crate::model::client::status::Status;
use std::time::Instant;

//use diesel;
//use diesel::prelude::*;
//use diesel::mysql::MysqlConnection;
//
//#[derive(Queryable)]
pub struct Client {
    pub id: i32,
    pub name: String,
    pub pwd: String,
    pub status: Status,
    pub created_at: Instant,
    pub updated_at: Instant,
    creds: Vec<String>,
    extension: Box<dyn Extension>,
}

impl Client {
    pub fn new(ext: Box<dyn Extension>, name: String, pwd: String) -> Self {
        Client{
            id: 0,
            name: name,
            pwd: pwd,
            created_at: Instant::now(),
            updated_at: Instant::now(),
            status: Status::PENDING,
            creds: vec!{},
            extension: ext,
        }
    }
}

impl Controller for Client {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_status(&self) -> &Status {
        &self.status
    }

    fn get_addr(&self) -> &str {
        self.extension.get_addr()
    }
    
    fn match_pwd(&self, pwd: String) -> bool {
        self.pwd == pwd
    }
}