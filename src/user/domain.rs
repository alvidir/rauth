use std::error::Error;
use std::time::{Duration, SystemTime};

use crate::schema::users;
use crate::regex::*;

pub trait UserRepository {
    fn find(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(Clone)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub email: String,
    pub pwd: String,
    pub meta_id: i32,
}

impl User {
    pub fn new<'a>(email: &'a str, pwd: &'a str) -> Result<Self, Box<dyn Error>> {
        match_email(email)?;
        match_pwd(pwd)?;

        let user = User {
            id: 0,
            email: email.to_string(),
            pwd: pwd.to_string(),
            meta_id: 0,
        };

        Ok(user)
    }
}

pub trait SessionRepository {
    fn find(&self, cookie: &str) -> Result<Session, Box<dyn Error>>;
    fn save(&self, session: &mut Session) -> Result<(), Box<dyn Error>>;
    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>>;
}


pub struct Session {
    cookie: String,
    created_at: SystemTime,
    touch_at: SystemTime,
    deadline: SystemTime,
    user: User,
}

impl Session {
    pub fn new(user: User, cookie: String, timeout: Duration) -> Self {
        Session{
            cookie: cookie,
            created_at: SystemTime::now(),
            touch_at: SystemTime::now(),
            deadline: SystemTime::now() + timeout,
            user: user,
        }
    }
}