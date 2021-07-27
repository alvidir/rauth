use std::error::Error;
use crate::metadata::domain::{Metadata, MetadataRepository};
use crate::secret::domain::{Secret, SecretRepository};
use super::domain::{App, AppRepository};

pub fn app_register(app_repo: &dyn AppRepository,
                    secret_repo: &dyn SecretRepository,
                    meta_repo: &dyn MetadataRepository,
                    pem: &[u8],
                    url: &str) -> Result<(), Box<dyn Error>> {

    println!("got a register request for application {} ", url);
    
    let meta = Metadata::new(meta_repo)?;
    let secret = Secret::new(secret_repo, pem)?;
    App::new(app_repo, secret, meta, url)?;
    Ok(())
}

pub fn app_delete(app_repo: Box<dyn AppRepository>,
                  url: &str) -> Result<(), Box<dyn Error>> {
    
    println!("got a deletion request from application {} ", url);
    
    let app = app_repo.find(url)?;
    app_repo.delete(&app)?;
    Ok(())
}