use std::error::Error;
pub mod user;
pub mod app;
pub mod session;
pub mod secret;
pub mod enums;
pub mod namesp;

mod client;
mod dir;

pub trait Gateway {
    fn select(&mut self) -> Result<(), Box<dyn Error>>;
    fn insert(&mut self) -> Result<(), Box<dyn Error>>;
    fn update(&mut self) -> Result<(), Box<dyn Error>>;
    fn delete(&self) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use openssl::encrypt::Decrypter;
    use openssl::sign::Signer;
    use openssl::hash::MessageDigest;
    use openssl::pkey::PKey;
    use openssl::rsa::{Rsa, Padding};
    use super::{enums, user, client, secret, app, session, namesp};
    use crate::default::tests::{get_prefixed_data, DUMMY_DESCR, DUMMY_PWD};

    #[test]
    fn client_new_ok() {
        const PREFIX: &str = "client_new_ok";

        let (name, _) = get_prefixed_data(PREFIX, false);
        let client = client::Client::new(enums::Kind::USER, &name).unwrap();

        use super::client::Ctrl;
        assert_eq!(client.get_id(), 0);
        assert_eq!(client.get_name(), name);
        assert_eq!(client.get_kind(), enums::Kind::USER);
    }

    #[test]
    fn client_new_name_ko() {
        const PREFIX: &str = "client_new_name_ko";

        let (name, _) = get_prefixed_data(PREFIX, false);
        let name = format!("#{}", name);
        assert!(client::Client::new(enums::Kind::USER, &name).is_err());
    }

    #[test]
    fn user_new_ok() {
        const PREFIX: &str = "user_new_ok";

        let (name, email) = get_prefixed_data(PREFIX, false);
        let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();

        use super::user::Ctrl;
        assert_eq!(user.get_id(), 0);
        assert_eq!(user.get_client_id(), 0);
        assert_eq!(user.get_name(), name);
        assert_eq!(user.get_email(), email);
        assert!(user.match_pwd(DUMMY_PWD));
    }

    #[test]
    fn user_new_name_ko() {
        const PREFIX: &str = "user_new_name_ko";

        let (name, email) = get_prefixed_data(PREFIX, false);
        let name = format!("#{}", name);
        assert!(user::User::new(&name, &email, DUMMY_PWD).is_err());
    }

    #[test]
    fn user_new_email_ko() {
        const PREFIX: &str = "user_new_email_ko";

        let (name, email) = get_prefixed_data(PREFIX, false);
        let email = format!("{}!", email);
        assert!(user::User::new(&name, &email, DUMMY_PWD).is_err());
    }

    #[test]
    fn user_new_pwd_ko() {
        const PREFIX: &str = "user_new_pwd_ko";

        let (name, email) = get_prefixed_data(PREFIX, false);
        let pwd = format!("{}G", DUMMY_PWD);
        assert!(user::User::new(&name, &email, &pwd).is_err());
    }

    #[test]
    fn user_match_pwd() {
        const PREFIX: &str = "user_match_pwd";

        let (name, email) = get_prefixed_data(PREFIX, false);
        let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();

        use super::user::Ctrl;
        assert!(!user.match_pwd(&format!("{}G", DUMMY_PWD)));
        assert!(user.match_pwd(DUMMY_PWD));
    }

    #[test]
    fn app_new_ok() {
        use super::app::Ctrl;
        const PREFIX: &str = "app_new_ok";

        let (name, url) = get_prefixed_data(PREFIX, true);
        let app = app::App::new(&name, &url, DUMMY_DESCR).unwrap();

        assert_eq!(app.get_id(), 0);
        assert_eq!(app.get_client_id(), 0);
        assert_eq!(app.get_name(), name);
        assert_eq!(app.get_url(), url);
        assert_eq!(app.get_descr(), DUMMY_DESCR);
    }

    #[test]
    fn app_new_name_ko() {
        const PREFIX: &str = "app_new_name_ko";

        let (name, url) = get_prefixed_data(PREFIX, true);
        let name = format!("#{}", name);
        assert!(app::App::new(&name, &url, DUMMY_DESCR).is_err());
    }

    #[test]
    fn app_new_url_ko() {
        const PREFIX: &str = "app_new_url_ko";

        let (name, url) = get_prefixed_data(PREFIX, true);
        let url = format!("{}!", url);
        assert!(app::App::new(&name, &url, DUMMY_DESCR).is_err());
    }

    #[test]
    fn secret_new_ok() {
        const PREFIX: &str = "secret_new_ok";
        
        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();

        let (name, _) = get_prefixed_data(PREFIX, false);
        let secret = secret::Secret::new(0, &name, &public).unwrap();

        use super::secret::Ctrl;
        assert_eq!(secret.get_client_id(), 0);
    }

    #[test]
    fn secret_new_name_ko() {
        const PREFIX: &str = "secret_new_name_ko";

        let (name, _) = get_prefixed_data(PREFIX, false);
        let name = format!("#{}", name);
        assert!(secret::Secret::new(0, &name, b"").is_err());
    }

    #[test]
    fn secret_new_pem_ko() {
        const PREFIX: &str = "secret_new_pem_ko";

        let (name, _) = get_prefixed_data(PREFIX, false);
        assert!(secret::Secret::new(0, &name, b"").is_err());
    }

    #[test]
    fn secret_verify() {
        const PREFIX: &str = "secret_verify";
        
        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();

        let (name, _) = get_prefixed_data(PREFIX, false);
        let secret = secret::Secret::new(0, &name, &public).unwrap();

        let data: &[u8] = b"hello world";
        let mut signer = Signer::new(MessageDigest::sha256(), &rsa).unwrap();
        signer.update(data).unwrap();

        let firm = signer.sign_to_vec().unwrap();

        use super::secret::Ctrl;
        let mut verifier = secret.get_verifier().unwrap();
        verifier.update(data).unwrap();

        assert!(verifier.verify(&firm).unwrap());
    }

    #[test]
    fn secret_encrypt() {
        const PREFIX: &str = "secret_encrypt";
        
        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();

        let (name, _) = get_prefixed_data(PREFIX, false);
        let secret = secret::Secret::new(0, &name, &public).unwrap();

        let data: &[u8] = b"hello world";

        use super::secret::Ctrl;
        let encrypted = secret.encrypt(data).unwrap();        
        
        // Decrypt the data
        let mut decrypter = Decrypter::new(&rsa).unwrap();
        decrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        
        // Create an output buffer
        let buffer_len = decrypter.decrypt_len(&encrypted).unwrap();
        let mut decrypted = vec![0; buffer_len];
        
        // Encrypt and truncate the buffer
        let decrypted_len = decrypter.decrypt(&encrypted, &mut decrypted).unwrap();
        decrypted.truncate(decrypted_len);
        assert_eq!(&*decrypted, data);
    }

    #[test]
    fn namesp_new_ok() {
        use super::app::Ctrl as AppCtrl;
        const PREFIX: &str = "namesp_new_ok";

        let (name, url) = get_prefixed_data(PREFIX, true);
        let app = app::App::new(&name, &url, DUMMY_DESCR).unwrap();
        let app_label = app.get_label().to_string();
        let app_id = app.get_id();

        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();

        let (name, _) = get_prefixed_data(PREFIX, false);
        let secret = secret::Secret::new(0, &name, &public).unwrap();

        let namesp = namesp::get_instance().new_namespace(app, secret).unwrap();
        assert_eq!(namesp.get_id(), app_id);
        assert_eq!(namesp.get_label(), app_label);

        assert!(namesp::get_instance().get_by_label(&app_label).is_some());
        assert!(namesp::get_instance().get_by_id(app_id).is_some());
    }

    #[test]
    fn namesp_destroy() {
        use super::app::Ctrl as AppCtrl;
        const PREFIX: &str = "namesp_destroy";

        let (name, url) = get_prefixed_data(PREFIX, true);
        let app = app::App::new(&name, &url, DUMMY_DESCR).unwrap();
        let app_label = app.get_label().to_string();

        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();

        let (name, _) = get_prefixed_data(PREFIX, false);
        let secret = secret::Secret::new(0, &name, &public).unwrap();

        assert!(namesp::get_instance().new_namespace(app, secret).is_ok());
        assert!(namesp::get_instance().destroy_namespace(&app_label).is_ok());
        assert!(namesp::get_instance().get_by_label(&app_label).is_none());
    }

    #[test]
    fn namesp_set_token() {
        use crate::token::Token;
        const PREFIX: &str = "namesp_set_token";

        let (name, url) = get_prefixed_data(PREFIX, true);
        let app = app::App::new(&name, &url, DUMMY_DESCR).unwrap();

        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();

        let (name, _) = get_prefixed_data(PREFIX, false);
        let secret = secret::Secret::new(0, &name, &public).unwrap();

        let np = namesp::get_instance().new_namespace(app, secret).unwrap();
        let want_cookie = Token::new(8);
        let want_token = want_cookie.clone();

        assert!(np.set_token(want_cookie.clone(), want_token.clone()).is_ok());
        let got_token = np.get_token(&want_cookie).unwrap();
        assert_eq!(got_token.to_string(), want_token.to_string());
    }


    #[test]
    fn namesp_delete_token() {
        use crate::token::Token;
        const PREFIX: &str = "namesp_delete_token";

        let (name, url) = get_prefixed_data(PREFIX, true);
        let app = app::App::new(&name, &url, DUMMY_DESCR).unwrap();

        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();

        let (name, _) = get_prefixed_data(PREFIX, false);
        let secret = secret::Secret::new(0, &name, &public).unwrap();

        let np = namesp::get_instance().new_namespace(app, secret).unwrap();
        let want_cookie = Token::new(8);
        let want_token = want_cookie.clone();

        assert!(np.set_token(want_cookie.clone(), want_token.clone()).is_ok());
        let got_token = np.delete_token(&want_cookie).unwrap();
        assert_eq!(got_token.as_str(), want_token.as_str());
        assert!(np.get_token(&want_cookie).is_none());
    }

    #[test]
    fn session_new_ok() {
        use user::Ctrl;
        use crate::proto::Status;

        const PREFIX: &str = "session_new_ok";
    
        let before = SystemTime::now();
        let (name, email) = get_prefixed_data(PREFIX, false);
        let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
        assert!(session::get_instance().get_by_email(user.get_email()).is_none());
        
        let user_id = user.get_id();
        let sess = session::get_instance().new_session(user).unwrap();
        let after = SystemTime::now();
    
        assert_eq!(sess.get_user_id(), user_id);
        assert!(before < sess.get_touch_at());
        assert!(after > sess.get_touch_at());  
        assert!(sess.is_alive().is_ok());
        assert!(sess.match_pwd(DUMMY_PWD));
        
        assert_eq!(sess.get_status(), Status::New);
        
        let cookie = sess.get_cookie();
        let sess = session::get_instance().get_by_cookie(cookie).unwrap();
        assert_eq!(sess.get_user_id(), user_id);
        
        let sess = session::get_instance().get_by_email(&email).unwrap();
        assert_eq!(sess.get_user_id(), user_id);
        
        let sess = session::get_instance().get_by_name(&name).unwrap();
        assert_eq!(sess.get_user_id(), user_id);
    }
    
    #[test]
    fn session_new_ko() {
        use user::Ctrl;
        const PREFIX: &str = "session_new_ko";
    
        let (name, email) = get_prefixed_data(PREFIX, false);
        let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
        assert!(session::get_instance().get_by_email(user.get_email()).is_none());
        assert!(session::get_instance().new_session(user).is_ok());
    
        let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
        assert!(session::get_instance().new_session(user).is_err());
    }
    
    #[test]
    fn session_destroy() {
        const PREFIX: &str = "session_destroy";
    
        let (name, email) = get_prefixed_data(PREFIX, false);
        let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
        
        let sess = session::get_instance().new_session(user).unwrap();
        let cookie = sess.get_cookie().clone(); // if not cloned memory address gets invalid due the owner session has been deleted
    
        assert!(session::get_instance().destroy_session(&cookie).is_ok());
        assert!(session::get_instance().get_by_cookie(&cookie).is_none());
        assert!(session::get_instance().get_by_name(&name).is_none());
        assert!(session::get_instance().get_by_email(&email).is_none());
    }

    #[test]
    fn session_new_directory() {
        use user::Ctrl;
        const PREFIX: &str = "session_new_directory";
    
        let (name, email) = get_prefixed_data(PREFIX, false);
        let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
        let user_id = user.get_id();
        
        let sess = session::get_instance().new_session(user).unwrap();
    
        let app_id = 0_i32;
        let token = sess.new_directory(app_id).unwrap();
        let dir = sess.get_directory(&token).unwrap();

        assert_eq!(dir.get_user_id(), user_id);
        assert_eq!(dir.get_app_id(), app_id);
    }

    #[test]
    fn session_delete_directory() {
        const PREFIX: &str = "session_delete_directory";
    
        let (name, email) = get_prefixed_data(PREFIX, false);
        let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
        let sess = session::get_instance().new_session(user).unwrap();
    
        let want_app_id = 0_i32;
        let token = sess.new_directory(want_app_id).unwrap();
        let got_app_id = sess.delete_directory(&token).unwrap();
        assert_eq!(got_app_id, want_app_id);
        assert!(sess.get_directory(&token).is_none());
    }
}