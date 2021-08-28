use std::error::Error;
use std::time::Duration;
use crate::metadata::domain::Metadata;
use crate::security;
use crate::constants::{errors, settings};
use crate::smtp;
use crate::session::{
    get_repository as get_sess_repository,
    domain::{Session, Token as SessionToken},
};

use crate::directory::get_repository as get_dir_repository;
use crate::secret::{
    get_repository as get_secret_repository,
    domain::Secret,
};
use super::{
    get_repository as get_user_repository,
    domain::{User, Token},
};

/// If, and only if, there is no user with the same email, a new user with these email and password is created into the system
pub fn user_signup(email: &str,
                   password: &str) -> Result<(), Box<dyn Error>> {
    
    info!("got a signup request from user {} ", email);
    
    let meta = Metadata::new();
    let mut user = User::new(meta, email, password)?;
    get_user_repository().create(&mut user)?;
    
    let claim = Token::new(&user, Duration::from_secs(settings::TOKEN_TIMEOUT));
    let token = security::encode_jwt(claim)?;
    
    smtp::send_verification_email(email, &token)?;
    Ok(())
}

/// If, and only if, the provided token is valid, the owner gets verified
pub fn user_verify(token: &str) -> Result<(), Box<dyn Error>> {

    info!("got a verification request for token {} ", token);

    let claim = security::decode_jwt::<Token>(token)?;
    let mut user = get_user_repository().find(claim.sub)?;
    user.verify()?;
    
    get_user_repository().save(&user)?;
    Ok(())
}

/// If, and only if, the provided credentials matches with the user's ones, the user and all its data is deleted
/// from the system and repositories
pub fn user_delete(email: &str,
                   pwd: &str,
                   totp: &str) -> Result<(), Box<dyn Error>> {
    
    info!("got a deletion request from user {} ", email);

    let user = get_user_repository().find_by_email(email)?;
    if !user.match_password(pwd) {
        return Err(errors::NOT_FOUND.into());
    }

    // if, and only if, the user has activated the 2fa
    if let Some(secret) = &user.secret {
        let data = secret.get_data();
        security::verify_totp(data, totp)?;
    }

    // if the user was logged in, the session must be removed
    if let Ok(sess_arc) = get_sess_repository().find_by_email(&user.email) {
        let sess = match sess_arc.write() {
            Ok(sess) => sess,
            Err(err) => {
                error!("read-write lock for session got poisoned: {}", err);
                return Err(errors::POISONED.into());
            }
        };

        get_sess_repository().delete(&sess)?; // do not save directories
    }

    get_dir_repository().delete_all_by_user(&user)?;
    get_user_repository().delete(&user)?;
    Ok(())
}

/// All available actions to apply over the 2FA method of a user
pub enum TfaActions {
    ENABLE,
    DISABLE
}

const TOTP_SECRET_PROPOSAL_KEY: &str = "user::totp_secret";

fn user_enable_two_factor_authenticator(sess: &mut Session, totp: &str) -> Result<String, Box<dyn Error>> {
    if sess.get_user().secret.is_some() {
        // if the 2FA is already enabled the actions must fail
        return Err(errors::HAS_FAILED.into());
    }

    match sess.get(TOTP_SECRET_PROPOSAL_KEY) {
        Some(proposal) => {
            let key = proposal.as_bytes();
            sess.remove(TOTP_SECRET_PROPOSAL_KEY);

            if security::verify_totp(key, totp).is_err() {
                return Err(errors::UNAUTHORIZED.into());
            }

            let mut new_secret = Secret::new(key);
            get_secret_repository().create(&mut new_secret)?;

            let old_secret = sess.get_user_mut().set_secret(Some(new_secret));
            if let Err(err) = get_user_repository().save(sess.get_user()) {
                // this line will not panic due the previous set of Secret
                let new_secret = sess.get_user_mut().set_secret(old_secret).unwrap();
                get_secret_repository().delete(&new_secret)?;
                return Err(err);
            }

            if let Some(secret) = old_secret {
                get_secret_repository().delete(&secret)?;
            }

            Ok("".into())
        },

        None => {
            let token = security::get_random_string(settings::TOKEN_LEN);
            sess.store(TOTP_SECRET_PROPOSAL_KEY, &token);
            Ok(token)

            // let issuer = match env::var(environment::APP_NAME) {
            //     Ok(app_name) => app_name,
            //     Err(_) => "".to_string(),
            // };
            
            // let uri = security::get_uri_format(token.as_bytes(), &issuer, sess.get_user().get_email())?;
            //Ok(uri)
        }
    }
}

fn user_disable_two_factor_authenticator(sess: &mut Session, totp: &str) -> Result<(), Box<dyn Error>> {
    if let Some(secret) = &sess.get_user().secret {
        // if the 2FA is enabled it must be confirmed before deletion
        let data = secret.get_data();
        security::verify_totp(data, totp)?;
    } else {
        return Err(errors::HAS_FAILED.into());
    }

    // this block got duplicated in order to avoid mutability collisions
    if let Some(secret) = sess.get_user_mut().set_secret(None) {
        get_user_repository().save(sess.get_user())?;
        get_secret_repository().delete(&secret)?;
    }
    
    Ok(())
}

/// If, and only if, the provided token is valid and the provided credentials matches with the user's ones then a 2FA secret
/// is created or destroyed depending on the requested action.
pub fn user_two_factor_authenticator(token: &str,
                                     pwd: &str,
                                     totp: &str,
                                     action: TfaActions) -> Result<String, Box<dyn Error>> {

    info!("got an authentication method update for cookie {} ", token);
    let claim = security::decode_jwt::<SessionToken>(token)?;
    
    // session is required in order to have an ephimeral place where to find the metadata for the action
    // aka: sandbox
    let sess_arc = get_sess_repository().find(&claim.sub)?;
    let mut sess = match sess_arc.write() {
        Ok(sess) => sess,
        Err(err) => {
            error!("read-write lock for session got poisoned: {}", err);
            return Err(errors::POISONED.into());
        }
    };

    if !sess.get_user().match_password(pwd) {
        return Err(errors::NOT_FOUND.into());
    } else if !sess.get_user().is_verified() {
        return Err(errors::NOT_VERIFIED.into());
    }

    match action {
        TfaActions::ENABLE => user_enable_two_factor_authenticator(&mut sess, totp),
        TfaActions::DISABLE => {
            user_disable_two_factor_authenticator(&mut sess, totp)?;
            Ok("".into())
        },
    }
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
    use crate::secret::get_repository as get_secret_repository;
    use crate::metadata::get_repository as get_meta_repository;
    use crate::directory::get_repository as get_dir_repository;

    use crate::session::{
        application as sess_application,
    };

    use crate::app::{
        application as app_application,
        get_repository as get_app_repository,
    };

    use super::{
        user_signup,
        user_verify,
        user_delete,
        user_two_factor_authenticator,
        TfaActions
    };

    use super::super::{
        domain::Token,
        get_repository as get_user_repository,
    };


    const EC_SECRET: &[u8] = b"LS0tLS1CRUdJTiBFQyBQUklWQVRFIEtFWS0tLS0tCk1IY0NBUUVFSUlPejlFem04Ri9oSnluNTBrM3BVcW5Dc08wRVdGSjAxbmJjWFE1MFpyV0pvQW9HQ0NxR1NNNDkKQXdFSG9VUURRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaDYrbkVLbzFUOWdlcHd0UFR3MnpYNTRabgpkZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgRUMgUFJJVkFURSBLRVktLS0tLQo";
    const EC_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaAo2K25FS28xVDlnZXB3dFBUdzJ6WDU0Wm5kZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg";
    const PASSWORD: &str = "936a185caaa266bb9cbe981e9e05cb78cd732b0b3280eb944412bb6f8f8f07af";

    #[test]
    fn user_signup_should_not_fail() {
        dotenv::dotenv().unwrap();

        const EMAIL: &str = "user_signup_should_not_fail@testing.com";

        assert!(user_signup(EMAIL, PASSWORD).is_ok());

        let user = get_user_repository().find_by_email(EMAIL).unwrap();
        get_meta_repository().find(user.meta.get_id()).unwrap();

        get_user_repository().delete(&user).unwrap();
    }

    #[test]
    fn user_signup_repeated_should_fail() {
        dotenv::dotenv().unwrap();

        const EMAIL: &str = "user_signup_repeated_should_fail@testing.com";

        user_signup(EMAIL, PASSWORD).unwrap();
        let user = get_user_repository().find_by_email(EMAIL).unwrap();
        assert!(user_signup(EMAIL, PASSWORD).is_err());

        get_user_repository().delete(&user).unwrap();
    }

    #[test]
    fn user_verify_should_not_fail() {
        dotenv::dotenv().unwrap();

        const EMAIL: &str = "user_verify_should_not_fail@testing.com";

        user_signup(EMAIL, PASSWORD).unwrap();
        let user = get_user_repository().find_by_email(EMAIL).unwrap();

        let claim = Token::new(&user, Duration::from_secs(settings::TOKEN_TIMEOUT));
        let token = security::encode_jwt(claim).unwrap();

        assert!(user_verify(&token).is_ok());
        
        let user = get_user_repository().find_by_email(EMAIL).unwrap(); // get the updated data of the user
        assert!(user.verified_at.is_some());

        get_user_repository().delete(&user).unwrap();
    }

    #[test]
    fn user_delete_should_not_fail() {
        dotenv::dotenv().unwrap();

        const EMAIL: &str = "user_delete_should_not_fail@testing.com";

        user_signup(EMAIL, PASSWORD).unwrap();
        let user = get_user_repository().find_by_email(EMAIL).unwrap();

        assert!(user_delete(EMAIL, PASSWORD, "").is_ok());
        assert!(get_user_repository().find(user.id).is_err());
        assert!(get_meta_repository().find(user.meta.get_id()).is_err());
    }

    #[test]
    fn user_delete_with_wrong_password_should_fail() {
        dotenv::dotenv().unwrap();

        const EMAIL: &str = "user_delete_with_wrong_password_should_fail@testing.com";

        user_signup(EMAIL, PASSWORD).unwrap();
        let user = get_user_repository().find_by_email(EMAIL).unwrap();


        assert!(user_delete(EMAIL, "fakepassword", "").is_err());
        get_user_repository().delete(&user).unwrap();
    }

    #[test]
    fn user_tfa_enable_should_not_fail() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://user.tfa.enable.should.not.fail";
        const EMAIL: &str = "user_tfa_enable_should_not_fail@testing.com";

        let private = base64::decode(EC_SECRET).unwrap();
        let eckey = EcKey::private_key_from_pem(&private).unwrap();
        let keypair = PKey::from_ec_key(eckey).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        signer.update(EC_PUBLIC).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_application::app_register(URL, EC_PUBLIC, &signature).unwrap();
        let app = get_app_repository().find_by_url(URL).unwrap();

        user_signup(EMAIL, PASSWORD).unwrap();

        let user = get_user_repository().find_by_email(EMAIL).unwrap();
        let claim = Token::new(&user, Duration::from_secs(settings::TOKEN_TIMEOUT));
        let token = security::encode_jwt(claim).unwrap();
        user_verify(&token).unwrap();

        let token = sess_application::session_login(EMAIL, PASSWORD, "", URL).unwrap();
        
        // generate secret proposal
        let secret = user_two_factor_authenticator(&token, PASSWORD, "", TfaActions::ENABLE).unwrap();

        assert_ne!("", secret);
        assert!(user.secret.is_none());

        let code = security::generate_totp(secret.as_bytes())
            .unwrap()
            .generate();

        assert_eq!(code.len(), 6);

        // confirm secret
        let secret = user_two_factor_authenticator(&token, PASSWORD, &code, TfaActions::ENABLE).unwrap();
        assert_eq!("", secret);

        let user_id: i32 = user.get_id();
        let user = get_user_repository().find(user_id).unwrap(); // get the user up to date
        assert!(user.secret.is_some());

        let secret_id: i32 = user.secret.unwrap().get_id(); 
        assert!(get_secret_repository().find(secret_id).is_ok());

        
        // clear up data
        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_application::app_delete(URL, &signature).unwrap();
        
        assert!(user_delete(EMAIL, PASSWORD, "").is_err());
        assert!(user_delete(EMAIL, PASSWORD, &code).is_ok());

        assert!(get_dir_repository().find_by_user_and_app(user_id, app.get_id()).is_err());
        assert!(get_secret_repository().find(secret_id).is_err());
        
    }

    #[test]
    fn user_tfa_disable_should_not_fail() {
        dotenv::dotenv().unwrap();

        const URL: &str = "http://user.tfa.disable.should.not.fail";
        const EMAIL: &str = "user_tfa_disable_should_not_fail@testing.com";

        let private = base64::decode(EC_SECRET).unwrap();
        let eckey = EcKey::private_key_from_pem(&private).unwrap();
        let keypair = PKey::from_ec_key(eckey).unwrap();

        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        signer.update(EC_PUBLIC).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_application::app_register(URL, EC_PUBLIC, &signature).unwrap();
        let app = get_app_repository().find_by_url(URL).unwrap();

        user_signup(EMAIL, PASSWORD).unwrap();

        let user = get_user_repository().find_by_email(EMAIL).unwrap();
        let claim = Token::new(&user, Duration::from_secs(settings::TOKEN_TIMEOUT));
        let token = security::encode_jwt(claim).unwrap();
        user_verify(&token).unwrap();

        let token = sess_application::session_login(EMAIL, PASSWORD, "", URL).unwrap();

        // generate secret proposal
        let secret = user_two_factor_authenticator(&token, PASSWORD, "", TfaActions::ENABLE).unwrap();
        let code = security::generate_totp(secret.as_bytes())
            .unwrap()
            .generate();

        // confirm secret
        user_two_factor_authenticator(&token, PASSWORD, &code, TfaActions::ENABLE).unwrap();
        
        let user_id: i32 = user.get_id();
        let user = get_user_repository().find(user_id).unwrap(); // get the user up to date
        let secret_id: i32 = user.secret.unwrap().get_id();

        // disable tfa
        let secret = user_two_factor_authenticator(&token, PASSWORD, &code, TfaActions::DISABLE).unwrap();
        assert_eq!(secret, "");
        
        let user = get_user_repository().find(user_id).unwrap(); // get the user up to date
        assert!(user.secret.is_none());
        assert!(get_secret_repository().find(secret_id).is_err());

        // clear up data
        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(URL.as_bytes()).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        app_application::app_delete(URL, &signature).unwrap();
        assert!(user_delete(EMAIL, PASSWORD, "").is_ok());
        assert!(get_dir_repository().find_by_user_and_app(user_id, app.get_id()).is_err());
    }
}