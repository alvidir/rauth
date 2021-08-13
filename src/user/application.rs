use std::error::Error;
use std::time::Duration;
use crate::metadata::domain::Metadata;
use crate::security;
use crate::constants::{errors, settings};
use crate::smtp;
use crate::session::{
    get_repository as get_sess_repository,
    application::get_writable_session,
    domain::{Session, Token as SessionToken},
};

use crate::directory::get_repository as get_dir_repository;
use crate::secret::domain::Secret;
use super::domain::{User, Token};

/// If there is no user with the same email, a new user with these email and password is created into the system
pub fn user_signup(email: &str,
                   password: &str) -> Result<(), Box<dyn Error>> {
    
    info!("got a signup request from user {} ", email);
    
    let meta = Metadata::new()?;
    let user = User::new(meta, email, password)?;
    
    // the user will not be able to log in until they have verified their email
    let claim = Token::new(&user, Duration::from_secs(settings::TOKEN_TIMEOUT));
    let token = security::encode_jwt(claim)?;
    smtp::send_verification_email(email, &token)?;

    Ok(())
}

/// If, and only if, the provided token is valid, the owner gets verified
pub fn user_verify(token: &str) -> Result<(), Box<dyn Error>> {

    info!("got a verification request for token {} ", token);

    let claim = security::decode_jwt::<Token>(token)?;
    let mut user = super::get_repository().find(claim.sub)?;
    user.verify()?;
    user.save()?;

    Ok(())
}

/// If, and only if, the provided credentials matches with the user's ones, the user and all its data is deleted
/// from the system and repositories
pub fn user_delete(email: &str,
                   pwd: &str,
                   totp: &str) -> Result<(), Box<dyn Error>> {
    
    info!("got a deletion request from user {} ", email);

    let user = super::get_repository().find_by_email(email)?;
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
        let mut sess = get_writable_session(&sess_arc)?;
        sess.delete()?; // do not save directories
    }

    // delete all directories
    get_dir_repository().delete_all_by_user(&user)?;
    
    user.delete()?;
    Ok(())
}

/// All available actions to apply over the 2FA method of a user
pub enum TfaActions {
    ENABLE,
    DISABLE
}

const TOTP_SECRET_PROPOSAL_KEY: &str = "user::totp::secret_proposal";

fn user_enable_two_factor_authenticator(sess: &mut Session, totp: &str) -> Result<String, Box<dyn Error>> {
    if sess.get_user().secret.is_some() {
        // if the 2FA is already enabled the actions must fail
        return Err(errors::HAS_FAILED.into());
    }

    match sess.get(TOTP_SECRET_PROPOSAL_KEY) {
        Some(proposal) => {
            let key = proposal.as_bytes();
            
            // delete the proposal of secret from the sandbox
            sess.remove(TOTP_SECRET_PROPOSAL_KEY);

            if security::verify_totp(key, totp).is_ok() {
                let secret = Secret::new(key)?;
                sess.get_user_mut().update_secret(Some(secret))?;
                Ok("".into())
            } else {
                Err(errors::UNAUTHORIZED.into())
            }
        },

        None => {
            let token = security::get_random_string(settings::TOKEN_LEN);
            sess.store(TOTP_SECRET_PROPOSAL_KEY, &token);
            Ok(token)
        }
    }
}

fn user_disable_two_factor_authenticator(sess: &mut Session, totp: &str) -> Result<(), Box<dyn Error>> {
    if let Some(secret) = &sess.get_user().secret {
        // if the 2FA is enabled it must be confirmed before deletion
        let data = secret.get_data();
        security::verify_totp(data, totp)?;
    } else {
        // if the 2FA is already disabled the actions must fail
        return Err(errors::HAS_FAILED.into());
    }

    // in order to disable the 2FA method its secret must be removed from everywhere
    sess.get_user_mut().update_secret(None)?;
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
    let mut sess = get_writable_session(&sess_arc)?;

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