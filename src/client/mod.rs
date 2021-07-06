use std::error::Error;
pub mod framework;
mod application;
mod domain;

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
}