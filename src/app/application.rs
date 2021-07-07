use std::error::Error;
use super::domain::App;

pub fn app_register(url: &str, secret_id: &str) -> Result<String, Box<dyn Error>> {
    println!("Got a register request for application {} ", url);
    Err("Unimplemented".into())
}

pub fn app_delete(label: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a deletion request from application {} ", label);
    Err("Unimplemented".into())
}