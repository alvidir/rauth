use crate::model::session::traits::*;
use crate::model::session::status::Status;
use crate::model::client::traits::Controller as ClientController;

use std::time::{Duration, Instant};

//use diesel;
//use diesel::prelude::*;
//use diesel::mysql::MysqlConnection;
//
//#[derive(Queryable)]
pub struct Session {
    pub id: i32,
    pub cookie: String,
    pub created_at: Instant,
    pub touch_at: Instant,
    pub timeout: Duration,
    pub status: Status,
    client: Box<dyn ClientController>,
}

impl Session {
    pub fn new(client: Box<dyn ClientController>, cookie: String, timeout: Duration) -> Self {
        Session{
            id: 0,
            cookie: cookie,
            created_at: Instant::now(),
            touch_at: Instant::now(),
            timeout: timeout,
            status: Status::NEW,
            client: client,
        }
    }
}

impl Controller for Session {
    fn get_created_at(&self) -> Instant {
        self.created_at
    }

    fn get_touch_at(&self) -> Instant {
        self.touch_at
    }

    fn get_deadline(&self) -> Instant {
        self.created_at + self.timeout
    }

    fn get_status(&self) -> &Status {
        &self.status
    }

    fn get_cookie(&self) -> &str {
        &self.cookie
    }

    fn match_cookie(&self, cookie: String) -> bool {
        self.cookie == cookie
    }
}