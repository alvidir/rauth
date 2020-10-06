use crate::model::user::{user::UserImplementation,
                        traits::User};

pub fn new_user(name: &'static str, email: &'static str) -> Box<dyn User> {
    Box::new(UserImplementation::new(name, email))
}