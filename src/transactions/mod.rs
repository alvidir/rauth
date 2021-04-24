pub mod login;
pub mod logout;
pub mod signup;
pub mod delete_user;
pub mod delete_app;
pub mod register;
pub mod ticket;
pub mod resolve;

#[cfg(test)]
mod tests {
    use crate::token::Token;
use crate::transactions::{signup, delete_user, register};
    use crate::models::{user, secret, app, session, Gateway};
    use openssl::sign::Signer;
    use openssl::encrypt::Decrypter;
    use openssl::rsa::{Rsa, Padding};
    use openssl::hash::MessageDigest;
    use openssl::pkey::PKey;
    use crate::default::tests::{get_prefixed_data, DUMMY_DESCR, DUMMY_PWD};
    use crate::default;

    #[test]
    fn signup() {
        crate::initialize();
        const PREFIX: &str = "signup";

        let (name, email) = get_prefixed_data(PREFIX, false);

        // Signing up the user
        let tx_dummy = signup::TxSignup::new(&name, &email, DUMMY_PWD);
        tx_dummy.execute().unwrap();
        
        // Checking the user data
        let user_stream = user::find_by_email(&email).unwrap();
        let user: Box<&dyn user::Ctrl> = Box::new(user_stream.as_ref());
        assert_eq!(user.get_email(), email);

        // Checking the client data
        // let client_id = user.get_client_id();
        // let client: client::Wrapper = client::find_by_id(client_id).unwrap();
        // assert_eq!(client.get_name(), name);

        // Making sure a new user with the same data cannot be registered
        assert!(tx_dummy.execute().is_err());

        // Deleting the user and client
        let user_gw: Box<&dyn Gateway> = Box::new(user_stream.as_ref());
        user_gw.delete().unwrap();
    }

    #[test]
    fn delete_by_email() {
        crate::initialize();
        const PREFIX: &str = "delete_by_email";
        
        let (name, email) = get_prefixed_data(PREFIX, false);
        
        // Signing up the user
        let tx_signup = signup::TxSignup::new(&name, &email, DUMMY_PWD);
        tx_signup.execute().unwrap();
        
        let _client_id = {
            let user_stream = user::find_by_email(&email).unwrap();
            let user: Box<&dyn user::Ctrl> = Box::new(user_stream.as_ref());
            user.get_client_id()
        };
        
        // Delete the user
        let tx_dummy = delete_user::TxDelete::new(&email, DUMMY_PWD);
        tx_dummy.execute().unwrap();
        
        // Checking the user data
        assert!(user::find_by_email(&email).is_err());
        // assert!(client::find_by_id(client_id).is_err());
    }

    #[test]
    fn delete_by_name() {
        crate::initialize();
        const PREFIX: &str = "delete_by_name";
        
        let (name, email) = get_prefixed_data(PREFIX, false);
        
        // Signing up the user
        let tx_signup = signup::TxSignup::new(&name, &email, DUMMY_PWD);
        tx_signup.execute().unwrap();
        
        let _client_id = {
            let user_stream = user::find_by_email(&email).unwrap();
            let user: Box<&dyn user::Ctrl> = Box::new(user_stream.as_ref());
            user.get_client_id()
        };
        
        // Delete the user
        let tx_dummy = delete_user::TxDelete::new(&name, DUMMY_PWD);
        tx_dummy.execute().unwrap();
        
        // Checking the user data
        assert!(user::find_by_email(&email).is_err());
        // assert!(client::find_by_id(client_id).is_err());
    }

    #[test]
    fn register() {
        crate::initialize();
        const PREFIX: &str = "register";
        
        let (name, url) = get_prefixed_data(PREFIX, true);
    
        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();
        
        let mut signer = Signer::new(MessageDigest::sha256(), &rsa).unwrap();
        signer.update(name.as_bytes()).unwrap();
        signer.update(url.as_bytes()).unwrap();
        signer.update(DUMMY_DESCR.as_bytes()).unwrap();
        signer.update(&public).unwrap();
        
        let firm = signer.sign_to_vec().unwrap();
    
        // Register app
        let tx_register = register::TxRegister::new(&name, &url, DUMMY_DESCR, &public, &firm);
        let resp = tx_register.execute().unwrap();
    
         // Decrypt the data
        let mut decrypter = Decrypter::new(&rsa).unwrap();
        decrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        // Create an output buffer
        let buffer_len = decrypter.decrypt_len(&resp.label).unwrap();
        let mut decrypted = vec![0; buffer_len];
        // Encrypt and truncate the buffer
        let decrypted_len = decrypter.decrypt(&resp.label, &mut decrypted).unwrap();
        decrypted.truncate(decrypted_len);

        // Checking the user data
        let label = String::from_utf8(decrypted).unwrap();
        let app_stream = app::find_by_label(&label).unwrap();
        let app: Box<&dyn app::Ctrl> = Box::new(app_stream.as_ref());
        assert_eq!(app.get_url(), url);
        assert_eq!(app.get_descr(), DUMMY_DESCR);
        
        // Checking the client data
        let client_id = app.get_client_id();
        // let client: client::Wrapper = client::find_by_id(client_id).unwrap();
        // assert_eq!(client.get_name(), name);
        
        // Checking there is a default secret for the app
        use crate::default::RSA_NAME;
        let secret = secret::find_by_client_and_name(client_id, RSA_NAME).unwrap();

        use crate::models::secret::Ctrl;
        assert_eq!(secret.get_client_id(), client_id);

        let mut verifier = secret.get_verifier().unwrap();
        verifier.update(name.as_bytes()).unwrap();
        verifier.update(url.as_bytes()).unwrap();
        verifier.update(DUMMY_DESCR.as_bytes()).unwrap();
        verifier.update(&public).unwrap();

        if !verifier.verify(&firm).unwrap() {
            panic!("Verifier has failed")
        }
    
        // Deleting the secret in order to avoid sql-exceptions when deleting the client
        let secret_gw: Box<&dyn Gateway> = Box::new(secret.as_ref());
        secret_gw.delete().unwrap();
    
        // Deleting the app and client
        let app_gw: Box<&dyn Gateway> = Box::new(app_stream.as_ref());
        app_gw.delete().unwrap();
    }

    #[test]
    fn login_by_email() {
        use app::Ctrl as AppCtrl;
        use user::Ctrl as UserCtrl;
        crate::initialize();
        const PREFIX: &str = "login_by_email";
    
        // Setting up the required client
        let (user_name, email) = get_prefixed_data(PREFIX, false);
        let (app_name, url) = get_prefixed_data(PREFIX, true);
        let tx_signup = signup::TxSignup::new(&user_name, &email, DUMMY_PWD);
        tx_signup.execute().unwrap();
    
        let user = user::find_by_name(&user_name).unwrap();
    
        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();
        
        let mut signer = Signer::new(MessageDigest::sha256(), &rsa).unwrap();
        signer.update(app_name.as_bytes()).unwrap();
        signer.update(url.as_bytes()).unwrap();
        signer.update(DUMMY_DESCR.as_bytes()).unwrap();
        signer.update(&public).unwrap();
        
        let firm = signer.sign_to_vec().unwrap();
    
        // Register app
        let tx_register = register::TxRegister::new(&app_name, &url, DUMMY_DESCR, &public, &firm);
        let resp = tx_register.execute().unwrap();

        // Decrypt the data
        let mut decrypter = Decrypter::new(&rsa).unwrap();
        decrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        // Create an output buffer
        let buffer_len = decrypter.decrypt_len(&resp.label).unwrap();
        let mut decrypted = vec![0; buffer_len];
        // Encrypt and truncate the buffer
        let decrypted_len = decrypter.decrypt(&resp.label, &mut decrypted).unwrap();
        decrypted.truncate(decrypted_len);
 
        // Checking the user data
        let label = String::from_utf8(decrypted).unwrap();
        let app = app::find_by_label(&label).unwrap();
        assert_eq!(app.get_label(), label);

        // Login the new client using its email
        let tx_dummy = super::login::TxLogin::new(&email, DUMMY_PWD, &label);
        let resp = tx_dummy.execute().unwrap();

        use crate::proto::user_proto::Status;
        let status_new = Status::New as i32;
        assert_eq!(resp.status, status_new);
        assert_eq!(resp.cookie.len(), 2 * default::TOKEN_LEN);

        use crate::token::Token;
        let cookie = &resp.cookie[..default::TOKEN_LEN];
        let token = &resp.cookie[default::TOKEN_LEN..];
        let cookie = Token::from_string(cookie);
        let token = Token::from_string(token);
        let sess = session::get_instance().get_by_cookie(&cookie).unwrap();
        assert!(sess.get_directory(&token).is_some());
        assert_eq!(sess.get_cookie().as_str(), cookie.as_str());
        let got_token = sess.get_token(app.get_id()).unwrap();
        assert_eq!(got_token.as_str(), token.as_str());
        assert_eq!(sess.get_user_id(), user.get_id());

        // Checking there is a default secret for the app
        let secret = secret::find_by_client_and_name(app.get_client_id(), default::RSA_NAME).unwrap();
    
        // Deleting the dummy user and its data from the database
        secret.delete().unwrap();
        // Deleting the app and client
        app.delete().unwrap();
        // Deleting the user and client
        user.delete().unwrap();
    }

    #[test]
    fn login_by_name() {
        use app::Ctrl;
        crate::initialize();
        const PREFIX: &str = "login_by_name";
    
        // Setting up the required client
        let (user_name, email) = get_prefixed_data(PREFIX, false);
        let (app_name, url) = get_prefixed_data(PREFIX, true);
        let tx_signup = signup::TxSignup::new(&user_name, &email, DUMMY_PWD);
        tx_signup.execute().unwrap();
    
        let user = user::find_by_name(&user_name).unwrap();
    
        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();
        
        let mut signer = Signer::new(MessageDigest::sha256(), &rsa).unwrap();
        signer.update(app_name.as_bytes()).unwrap();
        signer.update(url.as_bytes()).unwrap();
        signer.update(DUMMY_DESCR.as_bytes()).unwrap();
        signer.update(&public).unwrap();
        
        let firm = signer.sign_to_vec().unwrap();
    
        // Register app
        let tx_register = register::TxRegister::new(&app_name, &url, DUMMY_DESCR, &public, &firm);
        let resp = tx_register.execute().unwrap();

        // Decrypt the data
        let mut decrypter = Decrypter::new(&rsa).unwrap();
        decrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        // Create an output buffer
        let buffer_len = decrypter.decrypt_len(&resp.label).unwrap();
        let mut decrypted = vec![0; buffer_len];
        // Encrypt and truncate the buffer
        let decrypted_len = decrypter.decrypt(&resp.label, &mut decrypted).unwrap();
        decrypted.truncate(decrypted_len);
 
        // Checking the user data
        let label = String::from_utf8(decrypted).unwrap();
        let app = app::find_by_label(&label).unwrap();
        assert_eq!(app.get_label(), label);

        // Login the new client using its email
        let tx_dummy = super::login::TxLogin::new(&user_name, DUMMY_PWD, &label);
        assert!(tx_dummy.execute().is_ok());

        // Checking there is a default secret for the app
        let secret = secret::find_by_client_and_name(app.get_client_id(), default::RSA_NAME).unwrap();
    
        // Deleting the dummy user and its data from the database
        secret.delete().unwrap();
        // Deleting the app and client
        app.delete().unwrap();
        // Deleting the user and client
        user.delete().unwrap();
    }

    #[test]
    fn logout() {
        use app::Ctrl as AppCtrl;
        crate::initialize();
        const PREFIX: &str = "logout";
    
        // Setting up the required client
        let (user_name, email) = get_prefixed_data(PREFIX, false);
        let (app_name, url) = get_prefixed_data(PREFIX, true);
        let tx_signup = signup::TxSignup::new(&user_name, &email, DUMMY_PWD);
        tx_signup.execute().unwrap();
    
        let user = user::find_by_name(&user_name).unwrap();
    
        // Generate a keypair
        let rsa = Rsa::generate(2048).unwrap();
        let rsa = PKey::from_rsa(rsa).unwrap();
        let public = rsa.public_key_to_pem().unwrap();
        
        let mut signer = Signer::new(MessageDigest::sha256(), &rsa).unwrap();
        signer.update(app_name.as_bytes()).unwrap();
        signer.update(url.as_bytes()).unwrap();
        signer.update(DUMMY_DESCR.as_bytes()).unwrap();
        signer.update(&public).unwrap();
        
        let firm = signer.sign_to_vec().unwrap();
    
        // Register app
        let tx_register = register::TxRegister::new(&app_name, &url, DUMMY_DESCR, &public, &firm);
        let resp = tx_register.execute().unwrap();

        // Decrypt the data
        let mut decrypter = Decrypter::new(&rsa).unwrap();
        decrypter.set_rsa_padding(Padding::PKCS1).unwrap();
        // Create an output buffer
        let buffer_len = decrypter.decrypt_len(&resp.label).unwrap();
        let mut decrypted = vec![0; buffer_len];
        // Encrypt and truncate the buffer
        let decrypted_len = decrypter.decrypt(&resp.label, &mut decrypted).unwrap();
        decrypted.truncate(decrypted_len);
 
        // Checking the user data
        let label = String::from_utf8(decrypted).unwrap();
        let app = app::find_by_label(&label).unwrap();

        // Login the new client using its email
        let tx_dummy = super::login::TxLogin::new(&email, DUMMY_PWD, &label);
        let resp = tx_dummy.execute().unwrap();
        let tx_dummy = super::logout::TxLogout::new(&resp.cookie);
        assert!(tx_dummy.execute().is_ok());
        let cookie = Token::from_string(&resp.cookie[..default::TOKEN_LEN]);
        let sess = session::get_instance().get_by_cookie(&cookie).unwrap();
        let token = Token::from_string(&resp.cookie[default::TOKEN_LEN..]);
        assert!(sess.get_directory(&token).is_none());

        // Checking there is a default secret for the app
        let secret = secret::find_by_client_and_name(app.get_client_id(), default::RSA_NAME).unwrap();
    
        // Deleting the dummy user and its data from the database
        secret.delete().unwrap();
        // Deleting the app and client
        app.delete().unwrap();
        // Deleting the user and client
        user.delete().unwrap();
    }

    #[test]
    fn ticket_restore() {
        use crate::models::Gateway;

        crate::initialize();
        const PREFIX: &str = "ticket_restore";
        
        let (name, email) = get_prefixed_data(PREFIX, false);
        
        // Signing up the user
        let tx_signup = signup::TxSignup::new(&name, &email, DUMMY_PWD);
        tx_signup.execute().unwrap();
        
        let user = user::find_by_name(&name).unwrap();
        let tx_ticket = super::ticket::TxTicket::new(0, &email);
        let resp = tx_ticket.execute().unwrap();

        // Deleting the user and client
        user.delete().unwrap();
    }
}