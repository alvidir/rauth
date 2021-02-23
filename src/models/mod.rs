use std::error::Error;
pub mod user;
pub mod app;
pub mod session;
pub mod secret;
pub mod enums;

mod client;

pub trait Gateway {
    fn insert(&mut self) -> Result<(), Box<dyn Error>>;
    fn update(&mut self) -> Result<(), Box<dyn Error>>;
    fn delete(&self) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
mod tests {
    use crate::transactions::{signup, delete, login, register, DEFAULT_PUBL_RSA_NAME};
    use super::{user, client, secret, app};
    use openssl::sign::{Signer, Verifier};
    use openssl::encrypt::{Encrypter, Decrypter};
    use openssl::rsa::{Rsa, Padding};
    use openssl::hash::MessageDigest;
    use openssl::pkey::PKey;
    use super::client::Ctrl;

    static DUMMY_NAME: &str = "dummy";
    static DUMMY_EMAIL: &str = "dummy@testing.com";
    static DUMMY_PWD: &str = "0C4fe7eBbfDbcCBE";
    static DUMMY_URL: &str = "https://www.dummy.com";
    static DUMMY_APP: &str = "dummy_app";
    static DUMMY_DESCR: &str = "this is a dummy application";

    fn get_prefixed_data(subject: &str, is_app: bool) -> (String, String) {
        let name = format!("{}_{}", subject, DUMMY_NAME);
        let email = {
            if is_app {
                format!("{}/{}", DUMMY_URL, subject)
            } else {
                format!("{}_{}", subject, DUMMY_EMAIL)
            }
        };

        (name, email)
    }

    #[test]
    fn signup_test() {
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
        let client_id = user.get_client_id();
        let client: client::Wrapper = client::find_by_id(client_id).unwrap();
        assert_eq!(client.get_name(), name);

        // Making sure a new user with the same data cannot be registered
        assert!(tx_dummy.execute().is_err());

        // Deleting the user and client
        let user_gw: Box<&dyn super::Gateway> = Box::new(user_stream.as_ref());
        user_gw.delete().unwrap();
    }

    #[test]
    fn delete_by_email_test() {
        crate::initialize();
        const PREFIX: &str = "delete_by_email";
        
        let (name, email) = get_prefixed_data(PREFIX, false);
        
        // Signing up the user
        let tx_signup = signup::TxSignup::new(&name, &email, DUMMY_PWD);
        tx_signup.execute().unwrap();
        
        let client_id = {
            let user_stream = user::find_by_email(&email).unwrap();
            let user: Box<&dyn user::Ctrl> = Box::new(user_stream.as_ref());
            user.get_client_id()
        };
        
        // Delete the user
        let tx_dummy = delete::TxDelete::new(&email, DUMMY_PWD);
        tx_dummy.execute().unwrap();
        
        // Checking the user data
        assert!(user::find_by_email(&email).is_err());
        assert!(client::find_by_id(client_id).is_err());
    }

    #[test]
    fn delete_by_name_test() {
        crate::initialize();
        const PREFIX: &str = "delete_by_name";
        
        let (name, email) = get_prefixed_data(PREFIX, false);
        
        // Signing up the user
        let tx_signup = signup::TxSignup::new(&name, &email, DUMMY_PWD);
        tx_signup.execute().unwrap();
        
        let client_id = {
            let user_stream = user::find_by_email(&email).unwrap();
            let user: Box<&dyn user::Ctrl> = Box::new(user_stream.as_ref());
            user.get_client_id()
        };
        
        // Delete the user
        let tx_dummy = delete::TxDelete::new(&name, DUMMY_PWD);
        tx_dummy.execute().unwrap();
        
        // Checking the user data
        assert!(user::find_by_email(&email).is_err());
        assert!(client::find_by_id(client_id).is_err());
    }

    //#[test]
    //fn login_by_email_test() {
    //    crate::initialize();
    //    const PREFIX: &str = "login_by_email";
    //
    //    // Setting up the required client
    //    let (name, email) = get_prefixed_data(PREFIX, false);
    //    let tx_signup = signup::TxSignup::new(&name, &email, DUMMY_PWD);
    //    tx_signup.execute().unwrap();
    //    
    //    // Login the new client using its email
    //    let tx_dummy = login::TxLogin::new(&email, DUMMY_PWD, DUMMY_APP);
    //    tx_dummy.execute().unwrap();
    //
    //    // Deleting the dummy user and its data from the database
    //    let secret_gw: Box<&dyn super::Gateway> = Box::new(secret_stream.as_ref());
    //    secret_gw.delete().unwrap();
    //
    //    let user_gw: Box<&dyn super::Gateway> = Box::new(user_stream.as_ref());
    //    user_gw.delete().unwrap();
    //}

    //#[test]
    //fn register_app_test() {
    //    crate::initialize();
    //    const PREFIX: &str = "register_app";
    //    
    //    let (name, url) = get_prefixed_data(PREFIX, true);
    //
    //    // Generate a keypair
    //    let rsa = Rsa::generate(2048).unwrap();
    //    let rsa = PKey::from_rsa(rsa).unwrap();
    //    let public = rsa.public_key_to_pem().unwrap();
    //    let public = String::from_utf8(public).unwrap();
    //    
    //    let mut signer = Signer::new(MessageDigest::sha256(), &rsa).unwrap();
    //    signer.update(name.as_bytes()).unwrap();
    //    signer.update(url.as_bytes()).unwrap();
    //    signer.update(DUMMY_DESCR.as_bytes()).unwrap();
    //    signer.update(public.as_bytes()).unwrap();
    //    
    //    let firm = signer.sign_to_vec().unwrap();
    //    let firm = String::from_utf8(firm).unwrap();
    //
    //    // Register app
    //    let tx_register = register::TxRegister::new(&name, &url, DUMMY_DESCR, &public, &firm);
    //    let resp = tx_register.execute().unwrap();
    //
    //    // Checking the user data
    //    let app_stream = app::find_by_label(&resp.label).unwrap();
    //    let app: Box<&dyn app::Ctrl> = Box::new(app_stream.as_ref());
    //    assert_eq!(app.get_url(), url);
    //    assert_eq!(app.get_descr(), DUMMY_DESCR);
    //    
    //    // Checking the client data
    //    let client_id = app.get_client_id();
    //    let client: client::Wrapper = client::find_by_id(client_id).unwrap();
    //    assert_eq!(client.get_name(), name);
    //    
    //    // Checking there is a default secret for the app
    //    let secret_stream = secret::find_by_client_and_name(client_id, DEFAULT_PUBL_RSA_NAME).unwrap();
    //    let secret: Box<&dyn secret::Ctrl> = Box::new(secret_stream.as_ref());
    //    assert_eq!(secret.get_client_id(), client_id);
    //
    //    // Deleting the secret in order to avoid sql-exceptions when deleting the client
    //    let secret_gw: Box<&dyn super::Gateway> = Box::new(secret_stream.as_ref());
    //    secret_gw.delete().unwrap();
    //
    //    // Deleting the app and client
    //    let app_gw: Box<&dyn super::Gateway> = Box::new(app_stream.as_ref());
    //    app_gw.delete().unwrap();
    //}
}