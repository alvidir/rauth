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
    use crate::transactions::signup;
    use super::{user, client, secret};
    use crate::transactions::DEFAULT_PKEY_NAME;

    static DUMMY_NAME: &str = "dummy";
    static DUMMY_EMAIL: &str = "dummy@testing.com";
    static DUMMY_PWD: &str = "0C4fe7eBbfDbcCBE52DC7A0DdF43bCaeEBaC0EE37bF03C4BAa0ed31eAA03d833";

    #[test]
    fn signup_test() {
        crate::initialize();
        const PREFIX: &str = "signup";

        let name = format!("{}_{}", PREFIX, DUMMY_NAME);
        let email = format!("{}_{}", PREFIX, DUMMY_EMAIL);

        let tx_dummy = signup::TxSignup::new(&name, &email, DUMMY_PWD);
        tx_dummy.execute().unwrap();
        
        let user_stream = user::find_by_email(&email, true).unwrap();
        let user: Box<&dyn user::Ctrl> = Box::new(user_stream.as_ref());
        assert_eq!(user.get_email(), email);

        let client_id = user.get_client_id();
        assert_eq!(user.get_email(), email);

        let client: Box<dyn client::Ctrl> = client::find_by_id(client_id, false).unwrap();
        assert_eq!(client.get_name(), name);

        assert!(tx_dummy.execute().is_err());

        let secret_stream = secret::find_by_client_and_name(client_id, DEFAULT_PKEY_NAME).unwrap();
        let secret: Box<&dyn user::Ctrl> = Box::new(user_stream.as_ref());
        assert_eq!(secret.get_client_id(), user.get_client_id());

        let secret_gw: Box<&dyn super::Gateway> = Box::new(secret_stream.as_ref());
        secret_gw.delete().unwrap();

        let user_gw: Box<&dyn super::Gateway> = Box::new(user_stream.as_ref());
        user_gw.delete().unwrap();
    }
}