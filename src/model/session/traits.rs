use std::time::Instant;
use crate::model::session::status::Status;

pub trait Controller {
    fn get_created_at(&self) -> Instant;
    fn get_touch_at(&self) -> Instant;
    fn get_deadline(&self) -> Instant;
    fn get_status(&self) -> &Status;
    fn get_cookie(&self) -> &str;
    fn match_cookie(&self, cookie: String) -> bool;
}