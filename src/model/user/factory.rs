use crate::model::user::{user::UserImplementation,
                        traits::User};

pub fn get_user(nickname: &'static str) -> Box<dyn User> {
    Box::new(UserImplementation::new(nickname, "addr"))
}