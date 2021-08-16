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
    
    let meta = Metadata::new();
    let secret = Secret::new(pem);

    App::new(secret, meta, url)?
        .insert()?;

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

#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod tests {
    use super::app_register;
    use super::super::get_repository;

    use openssl::sign::Signer;
    use openssl::pkey::{PKey};
    use openssl::ec::EcKey;
    use openssl::hash::MessageDigest;

    const EC_SECRET: &[u8] = b"LS0tLS1CRUdJTiBFQyBQUklWQVRFIEtFWS0tLS0tCk1IY0NBUUVFSUlPejlFem04Ri9oSnluNTBrM3BVcW5Dc08wRVdGSjAxbmJjWFE1MFpyV0pvQW9HQ0NxR1NNNDkKQXdFSG9VUURRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaDYrbkVLbzFUOWdlcHd0UFR3MnpYNTRabgpkZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgRUMgUFJJVkFURSBLRVktLS0tLQo";
    const EC_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaAo2K25FS28xVDlnZXB3dFBUdzJ6WDU0Wm5kZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg";

    #[test]
    fn app_register_ok() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://tests.app.register.ok";

        let private = base64::decode(EC_SECRET).unwrap();
        let eckey = EcKey::private_key_from_pem(&private).unwrap();
        let keypair = PKey::from_ec_key(eckey).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        signer.update(EC_PUBLIC).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_register(URL, EC_PUBLIC, &signature).unwrap();

        let app = get_repository().find_by_url(URL).unwrap();
        app.delete().unwrap();
    }
    
}