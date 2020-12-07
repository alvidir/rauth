use crate::model::client::status::Status;

pub trait Extension {
    fn get_addr(&self) -> &str;
}

pub trait Controller {
    fn get_status(&self) -> &Status;
    fn get_addr(&self) -> &str;
    fn get_id(&self) -> i32;
    fn match_pwd(&self, pwd: String) -> bool;
}