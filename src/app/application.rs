use std::error::Error;
use crate::metadata::domain::{Metadata, MetadataRepository};
use crate::secret::domain::{Secret, SecretRepository};
use crate::security;
use super::domain::{App, AppRepository};

pub fn app_register(app_repo: &dyn AppRepository,
                    secret_repo: &dyn SecretRepository,
                    meta_repo: &dyn MetadataRepository,
                    url: &str,
                    pem: &[u8],
                    firm: &[u8]) -> Result<(), Box<dyn Error>> {

    info!("got a register request for application {} ", url);

    let mut data: Vec<&[u8]> = Vec::new();
    data.push(url.as_bytes());
    data.push(pem);

    // the application can only be registered if, and only if, the provided secret matches
    // the message signature; otherwise there is no way to ensure the secret is the app's one
    security::verify_ec_signature(pem, firm, &data)?;
    
    let meta = Metadata::new(meta_repo)?;
    let secret = Secret::new(secret_repo, pem)?;
    App::new(app_repo, secret, meta, url)?;
    Ok(())
}

pub fn app_delete(app_repo: &dyn AppRepository,
                  url: &str,
                  firm: &[u8]) -> Result<(), Box<dyn Error>> {
    
    info!("got a deletion request from application {} ", url);

    let app = app_repo.find(url)?;
    let pem = app.secret.get_data();
    
    let mut data: Vec<&[u8]> = Vec::new();
    data.push(url.as_bytes());

    // in order to make sure the requester is the application itself the message's signature
    // must be checked
    security::verify_ec_signature(pem, firm, &data)?;
    
    app_repo.delete(&app)?;
    Ok(())
}