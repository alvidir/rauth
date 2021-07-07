pub mod framework;
mod application;
mod domain;

// #[cfg(test)]
// mod tests {
//     use std::time::SystemTime;
//     use openssl::encrypt::Decrypter;
//     use openssl::sign::Signer;
//     use openssl::hash::MessageDigest;
//     use openssl::pkey::PKey;
//     use openssl::rsa::{Rsa, Padding};
//     use super::{enums, user, client, secret, app, session, namesp};
//     use crate::default::tests::{get_prefixed_data, DUMMY_DESCR, DUMMY_PWD};

//     #[test]
//     fn user_new_ok() {
//         const PREFIX: &str = "user_new_ok";

//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();

//         use super::user::Ctrl;
//         assert_eq!(user.get_id(), 0);
//         assert_eq!(user.get_client_id(), 0);
//         assert_eq!(user.get_name(), name);
//         assert_eq!(user.get_email(), email);
//         assert!(user.match_pwd(DUMMY_PWD));
//     }

//     #[test]
//     fn user_new_name_ko() {
//         const PREFIX: &str = "user_new_name_ko";

//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let name = format!("#{}", name);
//         assert!(user::User::new(&name, &email, DUMMY_PWD).is_err());
//     }

//     #[test]
//     fn user_new_email_ko() {
//         const PREFIX: &str = "user_new_email_ko";

//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let email = format!("{}!", email);
//         assert!(user::User::new(&name, &email, DUMMY_PWD).is_err());
//     }

//     #[test]
//     fn user_new_pwd_ko() {
//         const PREFIX: &str = "user_new_pwd_ko";

//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let pwd = format!("{}G", DUMMY_PWD);
//         assert!(user::User::new(&name, &email, &pwd).is_err());
//     }

//     #[test]
//     fn user_match_pwd() {
//         const PREFIX: &str = "user_match_pwd";

//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();

//         use super::user::Ctrl;
//         assert!(!user.match_pwd(&format!("{}G", DUMMY_PWD)));
//         assert!(user.match_pwd(DUMMY_PWD));
//     }


//     #[test]
//     fn session_new_ok() {
//         use user::Ctrl;
//         use crate::proto::Status;

//         const PREFIX: &str = "session_new_ok";
    
//         let before = SystemTime::now();
//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
//         assert!(session::get_instance().get_by_email(user.get_email()).is_none());
        
//         let user_id = user.get_id();
//         let sess = session::get_instance().new_session(user).unwrap();
//         let after = SystemTime::now();
    
//         assert_eq!(sess.get_user_id(), user_id);
//         assert!(before < sess.get_touch_at());
//         assert!(after > sess.get_touch_at());  
//         assert!(sess.is_alive().is_ok());
//         assert!(sess.match_pwd(DUMMY_PWD));
        
//         assert_eq!(sess.get_status(), Status::New);
        
//         let cookie = sess.get_cookie();
//         let sess = session::get_instance().get_by_cookie(cookie).unwrap();
//         assert_eq!(sess.get_user_id(), user_id);
        
//         let sess = session::get_instance().get_by_email(&email).unwrap();
//         assert_eq!(sess.get_user_id(), user_id);
        
//         let sess = session::get_instance().get_by_name(&name).unwrap();
//         assert_eq!(sess.get_user_id(), user_id);
//     }
    
//     #[test]
//     fn session_new_ko() {
//         use user::Ctrl;
//         const PREFIX: &str = "session_new_ko";
    
//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
//         assert!(session::get_instance().get_by_email(user.get_email()).is_none());
//         assert!(session::get_instance().new_session(user).is_ok());
    
//         let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
//         assert!(session::get_instance().new_session(user).is_err());
//     }
    
//     #[test]
//     fn session_destroy() {
//         const PREFIX: &str = "session_destroy";
    
//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
        
//         let sess = session::get_instance().new_session(user).unwrap();
//         let cookie = sess.get_cookie().clone(); // if not cloned memory address gets invalid due the owner session has been deleted
    
//         assert!(session::get_instance().destroy_session(&cookie).is_ok());
//         assert!(session::get_instance().get_by_cookie(&cookie).is_none());
//         assert!(session::get_instance().get_by_name(&name).is_none());
//         assert!(session::get_instance().get_by_email(&email).is_none());
//     }

//     #[test]
//     fn session_new_directory() {
//         use user::Ctrl;
//         const PREFIX: &str = "session_new_directory";
    
//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
//         let user_id = user.get_id();
        
//         let sess = session::get_instance().new_session(user).unwrap();
    
//         let app_id = 0_i32;
//         let token = sess.new_directory(app_id).unwrap();
//         let dir = sess.get_directory(&token).unwrap();

//         assert_eq!(dir.get_user_id(), user_id);
//         assert_eq!(dir.get_app_id(), app_id);
//     }

//     #[test]
//     fn session_delete_directory() {
//         const PREFIX: &str = "session_delete_directory";
    
//         let (name, email) = get_prefixed_data(PREFIX, false);
//         let user = user::User::new(&name, &email, DUMMY_PWD).unwrap();
//         let sess = session::get_instance().new_session(user).unwrap();
    
//         let want_app_id = 0_i32;
//         let token = sess.new_directory(want_app_id).unwrap();
//         let got_app_id = sess.delete_directory(&token).unwrap();
//         assert_eq!(got_app_id, want_app_id);
//         assert!(sess.get_directory(&token).is_none());
//     }
// }