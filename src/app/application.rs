use std::error::Error;
use super::domain::AppRepository;

pub fn app_register(repo: impl AppRepository, url: &str, secret: &str) -> Result<String, Box<dyn Error>> {
    println!("Got a register request for application {} ", url);
    Err("Unimplemented".into())
}

pub fn app_delete(repo: impl AppRepository, label: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a deletion request from application {} ", label);
    Err("Unimplemented".into())
}