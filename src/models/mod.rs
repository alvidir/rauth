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
    use openssl::encrypt::Decrypter;
    use openssl::sign::Signer;
    use openssl::hash::MessageDigest;
    use openssl::pkey::PKey;
    use openssl::rsa::{Rsa, Padding};
    use super::{enums, user, client, secret, app};

    static DUMMY_NAME: &str = "dummy";
    static DUMMY_EMAIL: &str = "dummy@testing.com";
    static DUMMY_PWD: &str = "0C4fe7eBbfDbcCBE";
    static DUMMY_URL: &str = "dummy.com";
    static DUMMY_DESCR: &str = "this is a dummy application";

    fn get_prefixed_data(subject: &str, is_app: bool) -> (String, String) {
        let name = format!("{}_{}", subject, DUMMY_NAME);
        let addr = {
            if is_app {
                format!("http://{}.{}", subject, DUMMY_URL)
            } else {
                format!("{}_{}", subject, DUMMY_EMAIL)
            }
        };

        (name, addr)
    }

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
    fn user_pwd_match() {
        const PREFIX: &str = "user_pwd_match";

        let (name, email) = get_prefixed_data(PREFIX, false);
        let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();

        use super::user::Ctrl;
        assert!(!user.match_pwd(&format!("{}G", DUMMY_PWD)));
        assert!(user.match_pwd(DUMMY_PWD));
    }

    #[test]
    fn app_new_ok() {
        const PREFIX: &str = "app_new_ok";

        let (name, url) = get_prefixed_data(PREFIX, true);
        let app = app::App::new(&name, &url, DUMMY_DESCR).unwrap();

        use super::app::Ctrl;
        assert_eq!(app.get_id(), 0);
        assert_eq!(app.get_client_id(), 0);
        assert_eq!(app.get_name(), name);
        assert_eq!(app.get_url(), url);
        assert_eq!(app.get_descr(), DUMMY_DESCR);
    }

    #[test]
    fn app_new_name_ko() {
        const PREFIX: &str = "app_new_name_ko";

        let (name, url) = get_prefixed_data(PREFIX, false);
        let name = format!("#{}", name);
        assert!(user::User::new(&name, &url, DUMMY_PWD).is_err());
    }

    #[test]
    fn app_new_url_ko() {
        const PREFIX: &str = "user_new_url_ko";

        let (name, url) = get_prefixed_data(PREFIX, false);
        let url = format!("{}!", url);
        assert!(user::User::new(&name, &url, DUMMY_PWD).is_err());
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
}