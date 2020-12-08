use crate::model::client::Controller as ClientController;

use std::time::{Duration, Instant};

pub enum Status {
    ALIVE,
    DEAD,
    NEW
}

pub trait Controller {
    fn get_created_at(&self) -> Instant;
    fn get_touch_at(&self) -> Instant;
    fn get_deadline(&self) -> Instant;
    fn get_status(&self) -> &Status;
    fn get_cookie(&self) -> &str;
    fn match_cookie(&self, cookie: String) -> bool;
}

pub struct Session {
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