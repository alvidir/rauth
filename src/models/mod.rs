use std::error::Error;
pub mod user;
pub mod app;
pub mod session;
pub mod secret;

mod enums;
mod client;

pub trait Gateway {
    fn delete(&self) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
mod tests {
    use crate::transactions::{signup, delete};
    use super::{user, client, secret};
    use crate::transactions::DEFAULT_PKEY_NAME;

    static DUMMY_NAME: &str = "dummy";
    static DUMMY_EMAIL: &str = "dummy@testing.com";
    static DUMMY_PWD: &str = "0C4fe7eBbfDbcCBE52DC7A0DdF43bCaeEBaC0EE37bF03C4BAa0ed31eAA03d833";

    fn get_prefixed_data(prefix: &str) -> (String, String) {
        let name = format!("{}_{}", prefix, DUMMY_NAME);
        let email = format!("{}_{}", prefix, DUMMY_EMAIL);
        (name, email)
    }

    #[test]
    fn signup_test() {
        crate::initialize();
        const PREFIX: &str = "signup";

        let (name, email) = get_prefixed_data(PREFIX);

        // Signing up the user
        let tx_dummy = signup::TxSignup::new(&name, &email, DUMMY_PWD);
        tx_dummy.execute().unwrap();
        
        // Checking the user data
        let user_stream = user::find_by_email(&email, true).unwrap();
        let user: Box<&dyn user::Ctrl> = Box::new(user_stream.as_ref());
        assert_eq!(user.get_email(), email);

        // Checking the client data
        let client_id = user.get_client_id();
        let client: Box<dyn client::Ctrl> = client::find_by_id(client_id, false).unwrap();
        assert_eq!(client.get_name(), name);

        // Making sure a new user with the same data cannot be registered
        assert!(tx_dummy.execute().is_err());

        // Checking there is a default secret for the client
        let secret_stream = secret::find_by_client_and_name(client_id, DEFAULT_PKEY_NAME).unwrap();
        let secret: Box<&dyn secret::Ctrl> = Box::new(secret_stream.as_ref());
        assert_eq!(secret.get_client_id(), user.get_client_id());

        // Deleting the secret in order to avoid sql-exceptions when deleting the client
        let secret_gw: Box<&dyn super::Gateway> = Box::new(secret_stream.as_ref());
        secret_gw.delete().unwrap();

        // Deleting the user and client
        let user_gw: Box<&dyn super::Gateway> = Box::new(user_stream.as_ref());
        user_gw.delete().unwrap();
    }

    #[test]
    fn delete_test() {
        crate::initialize();
        const PREFIX: &str = "delete";
        
        let (name, email) = get_prefixed_data(PREFIX);
        
        // Signing up the user
        let tx_signup = signup::TxSignup::new(&name, &email, DUMMY_PWD);
        tx_signup.execute().unwrap();
        
        let client_id = {
            let user_stream = user::find_by_email(&email, true).unwrap();
            let user: Box<&dyn user::Ctrl> = Box::new(user_stream.as_ref());
            user.get_client_id()
        };
        
        // Delete the user
        let tx_dummy = delete::TxDelete::new(&name, DUMMY_PWD);
        tx_dummy.execute().unwrap();
        
        // Checking the user data
        assert!(user::find_by_email(&email, true).is_err());
        assert!(client::find_by_id(client_id, true).is_err());
        assert!(secret::find_by_client_and_name(client_id, DEFAULT_PKEY_NAME).is_err());
    }

    //#[test]
    //fn login_by_email_test() {
    //    crate::initialize();
    //    const PREFIX: &str = "login_by_email";
    //
    //    // Setting up the required client
    //    let (name, email) = get_prefixed_data(PREFIX);
    //    let tx_dummy = signup::TxSignup::new(&name, &email, DUMMY_PWD);
    //    tx_dummy.execute().unwrap();
    //    
    //    // Login the new client using its email
    //    let tx_dummy = login::TxLogin::new(&email, DUMMY_PWD);
    //    tx_dummy.execute().unwrap();
    //
    //    // Deleting the dummy user and its data from the database
    //    let secret_gw: Box<&dyn super::Gateway> = Box::new(secret_stream.as_ref());
    //    secret_gw.delete().unwrap();
    //
    //    let user_gw: Box<&dyn super::Gateway> = Box::new(user_stream.as_ref());
    //    user_gw.delete().unwrap();
    //}
}