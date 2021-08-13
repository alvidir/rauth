use std::error::Error;
use crate::metadata::domain::Metadata;
use crate::secret::domain::Secret;
use crate::security;
use crate::directory::get_repository as get_dir_repository;
use crate::session::get_repository as get_sess_repository;
use super::domain::App;

/// If, and only if, there is no application with the same url, a new app with these url and secret gets created into
/// the system
pub fn app_register(url: &str,
                    pem: &[u8],
                    firm: &[u8]) -> Result<(), Box<dyn Error>> {

    info!("got a register request for application {} ", url);

    let mut data: Vec<&[u8]> = Vec::new();
    data.push(url.as_bytes());
    data.push(pem);

    // the application can only be registered if, and only if, the provided secret matches
    // the message signature; otherwise there is no way to ensure the secret is the app's one
    security::verify_ec_signature(pem, firm, &data)?;
    
    let meta = Metadata::new()?;
    let secret = Secret::new(pem)?;
    App::new(secret, meta, url)?;
    Ok(())
}

/// If, and only if, the provided signature matches with the application secret, the app and all its data gets removed
/// from the system and repositories
pub fn app_delete(url: &str,
                  firm: &[u8]) -> Result<(), Box<dyn Error>> {
    
    info!("got a deletion request from application {} ", url);

    let app = super::get_repository().find_by_url(url)?;
    let pem = app.secret.get_data();
    
    let mut data: Vec<&[u8]> = Vec::new();
    data.push(url.as_bytes());

    // in order to make sure the requester is the application itself the message's signature
    // must be checked
    security::verify_ec_signature(pem, firm, &data)?;
    
    get_sess_repository().delete_all_by_app(&app)?;
    get_dir_repository().delete_all_by_app(&app)?;
    app.delete()?;
    Ok(())
}