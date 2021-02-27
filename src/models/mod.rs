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
    use super::{user, client, secret, app};

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
    fn user_new_ok() {
        crate::initialize();
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
        crate::initialize();
        const PREFIX: &str = "user_new_name_ko";

        let (name, email) = get_prefixed_data(PREFIX, false);
        let name = format!("{}!", name);
        assert!(user::User::new(&name, &email, DUMMY_PWD).is_err());
    }

    #[test]
    fn user_new_email_ko() {
        crate::initialize();
        const PREFIX: &str = "user_new_name_ko!";

        let (name, email) = get_prefixed_data(PREFIX, false);
        let email = format!("{}!", email);
        assert!(user::User::new(&name, &email, DUMMY_PWD).is_err());
    }

    #[test]
    fn user_new_pwd_ko() {
        crate::initialize();
        const PREFIX: &str = "user_new_name_ko!";

        let (name, email) = get_prefixed_data(PREFIX, false);
        let pwd = format!("{}G", DUMMY_PWD);
        assert!(user::User::new(&name, &email, &pwd).is_err());
    }
}