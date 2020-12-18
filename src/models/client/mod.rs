pub mod factory;

use std::time::SystemTime;
use self::factory::Gateway;

pub trait Extension {
    fn get_addr(&self) -> String;
}

pub trait Controller {
    fn get_status(&self) -> i32;
    fn get_addr(&self) -> String;
    fn get_id(&self) -> i32;
    fn match_pwd(&self, pwd: String) -> bool;
}

pub struct Client {
    data: Gateway,
    creds: Vec<String>,
    extension: Box<dyn Extension>,
}

impl Client {
    pub fn new(ext: Box<dyn Extension>, name: String, pwd: String) -> Self {
        Client{
            data: Gateway::new(name, pwd),
            creds: vec!{},
            extension: ext,
        }
    }

    fn build(gw: Gateway, ext: Box<dyn Extension>) -> Self {
        Client{
            data: gw,
            creds: vec!{},
            extension: ext,
        }
    }
}

impl Controller for Client {
    fn get_id(&self) -> i32 {
        self.data.id
    }

    fn get_status(&self) -> i32 {
        self.data.status_id
    }

    fn get_addr(&self) -> String {
        self.extension.get_addr()
    }
    
    fn match_pwd(&self, pwd: String) -> bool {
        self.data.pwd == pwd
    }
}