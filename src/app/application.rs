use std::error::Error;
use crate::metadata::domain::Metadata;
use crate::secret::domain::Secret;
use crate::security;
use crate::directory::get_repository as get_dir_repository;
use crate::session::get_repository as get_sess_repository;
use super::{
    get_repository as get_app_repository,
    domain::App,
};

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

    let mut app = App::new(secret, meta, url)?;
    get_app_repository().create(&mut app)?;
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

    security::verify_ec_signature(pem, firm, &data)?;
    
    get_sess_repository().delete_all_by_app(&app)?;
    get_dir_repository().delete_all_by_app(&app)?;
    get_app_repository().delete(&app)?;
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod tests {
    use openssl::sign::Signer;
    use openssl::pkey::{PKey};
    use openssl::ec::EcKey;
    use openssl::hash::MessageDigest;

    use crate::secret::get_repository as get_secret_repository;
    use crate::metadata::get_repository as get_meta_repository;

    use super::{app_register, app_delete};
    use super::super::get_repository as get_app_repository;

    const EC_SECRET: &[u8] = b"LS0tLS1CRUdJTiBFQyBQUklWQVRFIEtFWS0tLS0tCk1IY0NBUUVFSUlPejlFem04Ri9oSnluNTBrM3BVcW5Dc08wRVdGSjAxbmJjWFE1MFpyV0pvQW9HQ0NxR1NNNDkKQXdFSG9VUURRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaDYrbkVLbzFUOWdlcHd0UFR3MnpYNTRabgpkZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgRUMgUFJJVkFURSBLRVktLS0tLQo";
    const EC_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaAo2K25FS28xVDlnZXB3dFBUdzJ6WDU0Wm5kZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg";

    #[test]
    fn app_register_should_not_fail() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://app.register.should.not.fail";

        let private = base64::decode(EC_SECRET).unwrap();
        let eckey = EcKey::private_key_from_pem(&private).unwrap();
        let keypair = PKey::from_ec_key(eckey).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        signer.update(EC_PUBLIC).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_register(URL, EC_PUBLIC, &signature).unwrap();
        
        let app = get_app_repository().find_by_url(URL).unwrap();
        assert_eq!(URL, app.url);
        
        let secret = get_secret_repository().find(app.secret.get_id()).unwrap();
        assert_eq!(EC_PUBLIC, secret.get_data());

        get_meta_repository().find(app.meta.get_id()).unwrap();

        get_app_repository().delete(&app).unwrap();
    }

    #[test]
    fn app_register_repeated_should_fail() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://app.register.repeated.should.fail";

        let private = base64::decode(EC_SECRET).unwrap();
        let eckey = EcKey::private_key_from_pem(&private).unwrap();
        let keypair = PKey::from_ec_key(eckey).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        signer.update(EC_PUBLIC).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_register(URL, EC_PUBLIC, &signature).unwrap();
        assert!(app_register(URL, EC_PUBLIC, &signature).is_err());
        
        let app = get_app_repository().find_by_url(URL).unwrap();
        get_app_repository().delete(&app).unwrap();
    }

    #[test]
    fn app_register_wrong_signature_should_fail() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://app.register.wrong.signature.should.fail";

        assert!(app_register(URL, EC_PUBLIC, "fakesignature".as_bytes()).is_err());
        assert!(get_app_repository().find_by_url(URL).is_err());
    }

    #[test]
    fn app_delete_should_not_fail() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://app.delete.should.not.fail";

        let private = base64::decode(EC_SECRET).unwrap();
        let eckey = EcKey::private_key_from_pem(&private).unwrap();
        let keypair = PKey::from_ec_key(eckey).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        signer.update(EC_PUBLIC).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_register(URL, EC_PUBLIC, &signature).unwrap();
        let app = get_app_repository().find_by_url(URL).unwrap();
        
        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_delete(URL, &signature).unwrap();
        assert!(get_app_repository().find(app.id).is_err());
        assert!(get_secret_repository().find(app.secret.get_id()).is_err());
        assert!(get_meta_repository().find(app.meta.get_id()).is_err());
    }

    #[test]
    fn app_delete_wrong_signature_should_fail() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://app.delete.wrong.signature.should.fail";

        let private = base64::decode(EC_SECRET).unwrap();
        let eckey = EcKey::private_key_from_pem(&private).unwrap();
        let keypair = PKey::from_ec_key(eckey).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        signer.update(EC_PUBLIC).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_register(URL, EC_PUBLIC, &signature).unwrap();
        let app = get_app_repository().find_by_url(URL).unwrap();
        
        assert!(app_delete(URL, "fakesignature".as_bytes()).is_err());
        assert!(get_app_repository().find(app.id).is_ok());
        assert!(get_secret_repository().find(app.secret.get_id()).is_ok());
        assert!(get_meta_repository().find(app.meta.get_id()).is_ok());

        get_app_repository().delete(&app).unwrap();
    }
    
}