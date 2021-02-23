use std::error::Error;
use crate::models::{app, session, enums};
use crate::proto::app_proto::RegisterResponse;

pub struct TxRegister<'a> {
    name: &'a str,
    url: &'a str,
    descr: &'a str,
    public: &'a str,
    firm: &'a str
}

impl<'a> TxRegister<'a> {
    pub fn new(name: &'a str, url: &'a str, descr: &'a str, public: &'a str, firm: &'a str) -> Self {
        TxRegister{
            name: name,
            url: url,
            descr: descr,
            public: public,
            firm: firm,
        }
    }

    pub fn execute(&self) -> Result<RegisterResponse, Box<dyn Error>> {
        println!("Got a Register request for application {} ", self.name);
        
        let app: Box<dyn app::Ctrl> = app::App::new(self.name, self.url, self.descr)?;

        Ok(RegisterResponse{
            label: app.get_label().to_string(),
        })
    }
}