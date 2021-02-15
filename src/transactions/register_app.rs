use std::error::Error;
use crate::models::{app, admin, session, enums};

// Proto message structs
use crate::proto::dashboard_proto;
use dashboard_proto::RegisterAppResponse;

pub struct TxRegisterApp<'a> {
    cookie: &'a str,
    name: &'a str,
    url: &'a str,
}

impl<'a> TxRegisterApp<'a> {
    pub fn new(cookie: &'a str, name: &'a str, url: &'a str) -> Self {
        TxRegisterApp{
            cookie: cookie,
            name: name,
            url: url,
        }
    }

    pub fn execute(&self) -> Result<RegisterAppResponse, Box<dyn Error>> {
        println!("Got a RegisterApp request from user {} ", self.cookie);
        
        let sess = session::get_instance().get_session_by_cookie(self.cookie)?;
        let app: Box<dyn app::Ctrl> = app::App::new(self.name, self.url)?;
        let role_id = enums::Role::OWNER.to_int32();
        admin::Admin::new(sess.get_user_id(), app.get_id(), role_id)?;

        Ok(RegisterAppResponse{
            label: app.get_label().to_string(),
        })
    }
}