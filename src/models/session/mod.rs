pub mod provider;

use crate::models::client::Controller as ClientController;
use std::time::{Duration, SystemTime};

pub enum Status {
    ALIVE,
    DEAD,
    NEW
}

pub trait Controller {
    fn get_created_at(&self) -> SystemTime;
    fn get_touch_at(&self) -> SystemTime;
    fn get_deadline(&self) -> SystemTime;
    fn get_status(&self) -> &Status;
    fn get_cookie(&self) -> &str;
    fn get_client(&self) -> &Box<dyn ClientController>;
    fn match_cookie(&self, cookie: String) -> bool;
    fn get_addr(&self) -> String;
}

pub struct Session {
    pub cookie: String,
    pub created_at: SystemTime,
    pub touch_at: SystemTime,
    pub timeout: Duration,
    pub status: Status,
    client: Box<dyn ClientController>,
}

impl Session {
    pub fn new(client: Box<dyn ClientController>, cookie: &str, timeout: Duration) -> impl Controller {
        Session{
            cookie: cookie.to_string(),
            created_at: SystemTime::now(),
            touch_at: SystemTime::now(),
            timeout: timeout,
            status: Status::NEW,
            client: client,
        }
    }
}

impl Controller for Session {
    fn get_addr(&self) -> String {
        self.client.get_addr()
    }

    fn get_created_at(&self) -> SystemTime {
        self.created_at
    }

    fn get_touch_at(&self) -> SystemTime {
        self.touch_at
    }

    fn get_deadline(&self) -> SystemTime {
        self.created_at + self.timeout
    }

    fn get_status(&self) -> &Status {
        &self.status
    }

    fn get_cookie(&self) -> &str {
        &self.cookie
    }

    fn get_client(&self) -> &Box<dyn ClientController> {
        &self.client
    }

    fn match_cookie(&self, cookie: String) -> bool {
        self.cookie == cookie
    }
}