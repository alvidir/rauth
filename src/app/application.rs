use std::error::Error;
use crate::metadata::domain::MetadataRepository;
use crate::secret::domain::{Secret, SecretRepository};
use super::domain::{App, AppRepository};

pub trait SignatureManager {
    fn verify_signature(&self, pem: &[u8], signature: &[u8], data: &[&[u8]]) -> Result<(), Box<dyn Error>>;
}

pub fn app_register(app_repo: Box<dyn AppRepository>,
                    secret_repo: Box<dyn SecretRepository>,
                    meta_repo: Box<dyn MetadataRepository>,
                    pem: &[u8],
                    url: &str) -> Result<(), Box<dyn Error>> {

    println!("got a register request for application {} ", url);

    let secret = Secret::new(secret_repo, pem)?;
    App::new(app_repo, meta_repo, url, secret)?;
    Ok(())
}

pub fn app_delete(app_repo: Box<dyn AppRepository>,
                  url: &str) -> Result<(), Box<dyn Error>> {
    
    println!("got a deletion request from application {} ", url);
    
    let app = app_repo.find(url)?;
    app_repo.delete(&app)?;
    Ok(())
}