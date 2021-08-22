use std::error::Error;
use std::time::Duration;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use crate::user::get_repository as get_user_repository;
use crate::app::get_repository as get_app_repository;
use crate::directory::{
    get_repository as get_dir_repository,
    domain::Directory,
};

use crate::constants::{errors, settings};
use crate::security;

use super::{
    get_repository as get_sess_repository,
    domain::{Session, Token},
};

pub fn get_writable_session(sess_arc: &Arc<RwLock<Session>>) -> Result<RwLockWriteGuard<Session>, Box<dyn Error>> {
    match sess_arc.write() {
        Ok(sess) => Ok(sess),
        Err(err) => {
            error!("read-write lock for session got poisoned: {}", err);
            Err(errors::POISONED.into())
        }
    }
}

/// If, and only if, the provided credentials matches with the user's ones, a new directory is crated for the given
/// app (if not already exists) and a new token is generated
pub fn session_login(email: &str,
                     pwd: &str,
                     totp: &str,
                     app: &str) -> Result<String, Box<dyn Error>> {
    
    info!("got a login request from user {} ", email);

    // make sure the user exists and its credentials are alright
    let user = get_user_repository().find_by_email(email)?;
    if !user.match_password(pwd) {
        return Err(errors::NOT_FOUND.into());
    } else if !user.is_verified() {
        return Err(errors::NOT_VERIFIED.into());
    }

    // if, and only if, the user has activated the 2fa
    if let Some(secret) = &user.get_secret() {
        let data = secret.get_data();
        security::verify_totp(data, totp)?;
    }

    // get the existing session or create a new one
    let sess_arc = match get_sess_repository().find_by_email(email) {
        Ok(sess_arc) => sess_arc,
        Err(_) => {
            let timeout =  Duration::from_secs(settings::TOKEN_TIMEOUT);
            Session::new(user, timeout)?
        }
    };

    // generate a token for the gotten session and the given app
    let token = {
        let app = get_app_repository().find_by_url(app)?;
        let token: String;
        
        let mut sess = get_writable_session(&sess_arc)?;
        let claim = Token::new(&sess, &app, sess.deadline);
        token = security::encode_jwt(claim)?;

        if let None = sess.get_directory(&app) {
            if let Ok(dir) = get_dir_repository().find_by_user_and_app(sess.user.get_id(), app.get_id()) {
                sess.set_directory(dir)?;
            } else {
                let mut dir = Directory::new(&sess, &app);
                get_dir_repository().create(&mut dir)?;
                
                sess.set_directory(dir)?;
            }

            // subscribe the session's sid into the app's group 
            get_sess_repository().add_to_app_group(&app, &sess)?;
        }

        token
    };

    Ok(token)
}

/// If, and only if, the provided token is valid, the directory linked to it gets closed. If these was the latest
/// directory in the user's session, the whole session gets removed from the system
pub fn session_logout(token: &str) -> Result<(), Box<dyn Error>> {
    info!("got a logout request for cookie {} ", token);
    let claim = security::decode_jwt::<Token>(token)?;
    
    let sess_arc = get_sess_repository().find(&claim.sub)?;
    let mut sess = get_writable_session(&sess_arc)?;

    let app = get_app_repository().find(claim.app)?;
    if let Some(dir) = sess.delete_directory(&app) {
        // unsubscribe the session's from the app's group 
        get_sess_repository().delete_from_app_group(&app, &sess)?;
        get_dir_repository().save(&dir)?;
    }

    if sess.apps.len() == 0 {
        get_sess_repository().delete(&sess)?;
    }

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod tests {
    use std::time::Duration;
    use openssl::sign::Signer;
    use openssl::pkey::{PKey};
    use openssl::ec::EcKey;
    use openssl::hash::MessageDigest;

    use crate::constants::settings;
    use crate::security;
    use crate::directory::get_repository as get_dir_repository;
    
    use crate::user::{
        application as user_application,
        get_repository as get_user_repository,
        domain::Token,
    };
    
    use crate::app::{
        application as app_application,
        get_repository as get_app_repository,
    };

    use super::super::{
        application::{session_login, session_logout},
        get_repository as get_sess_repository,
    };


    const EC_SECRET: &[u8] = b"LS0tLS1CRUdJTiBFQyBQUklWQVRFIEtFWS0tLS0tCk1IY0NBUUVFSUlPejlFem04Ri9oSnluNTBrM3BVcW5Dc08wRVdGSjAxbmJjWFE1MFpyV0pvQW9HQ0NxR1NNNDkKQXdFSG9VUURRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaDYrbkVLbzFUOWdlcHd0UFR3MnpYNTRabgpkZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgRUMgUFJJVkFURSBLRVktLS0tLQo";
    const EC_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaAo2K25FS28xVDlnZXB3dFBUdzJ6WDU0Wm5kZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg";
    const PASSWORD: &str = "936a185caaa266bb9cbe981e9e05cb78cd732b0b3280eb944412bb6f8f8f07af";

    #[test]
    fn session_login_should_not_fail() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://session.login.should.not.fail";
        const EMAIL: &str = "session_login_should_not_fail@testing.com";

        let private = base64::decode(EC_SECRET).unwrap();
        let eckey = EcKey::private_key_from_pem(&private).unwrap();
        let keypair = PKey::from_ec_key(eckey).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        signer.update(EC_PUBLIC).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_application::app_register(URL, EC_PUBLIC, &signature).unwrap();
        let app = get_app_repository().find_by_url(URL).unwrap();

        user_application::user_signup(EMAIL, PASSWORD).unwrap();

        let user = get_user_repository().find_by_email(EMAIL).unwrap();
        let claim = Token::new(&user, Duration::from_secs(settings::TOKEN_TIMEOUT));
        let token = security::encode_jwt(claim).unwrap();
        user_application::user_verify(&token).unwrap();

        assert!(session_login(EMAIL, PASSWORD, "", URL).is_ok());
        assert!(get_sess_repository().find_by_email(EMAIL).is_ok());
        assert!(get_dir_repository().find_by_user_and_app(user.get_id(), app.get_id()).is_ok());

        // clear up data
        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_application::app_delete(URL, &signature).unwrap();
        user_application::user_delete(EMAIL, PASSWORD, "").unwrap();

        assert!(get_dir_repository().find_by_user_and_app(user.get_id(), app.get_id()).is_err());
    }

    #[test]
    fn session_logout_should_not_fail() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://session.logout.should.not.fail";
        const EMAIL: &str = "session_logout_should_not_fail@testing.com";

        let private = base64::decode(EC_SECRET).unwrap();
        let eckey = EcKey::private_key_from_pem(&private).unwrap();
        let keypair = PKey::from_ec_key(eckey).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        signer.update(EC_PUBLIC).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_application::app_register(URL, EC_PUBLIC, &signature).unwrap();
        let app = get_app_repository().find_by_url(URL).unwrap();

        user_application::user_signup(EMAIL, PASSWORD).unwrap();

        let user = get_user_repository().find_by_email(EMAIL).unwrap();
        let claim = Token::new(&user, Duration::from_secs(settings::TOKEN_TIMEOUT));
        let token = security::encode_jwt(claim).unwrap();
        user_application::user_verify(&token).unwrap();

        let token = session_login(EMAIL, PASSWORD, "", URL).unwrap();
        assert!(session_logout(&token).is_ok());
        assert!(get_sess_repository().find_by_email(EMAIL).is_err());
        assert!(get_dir_repository().find_by_user_and_app(user.get_id(), app.get_id()).is_ok());

        // clear up data
        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_application::app_delete(URL, &signature).unwrap();
        user_application::user_delete(EMAIL, PASSWORD, "").unwrap();

        assert!(get_dir_repository().find_by_user_and_app(user.get_id(), app.get_id()).is_err());
    }
}