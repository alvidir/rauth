#![allow(unused_must_use)]
use crate::proto::client_proto::SessionResponse;
use std::error::Error;
use crate::models::session;
use crate::transactions::signup::TxSignup;
use crate::transactions::login::TxLogin;

const DUMMY_NAME: &str = "dummy";
const DUMMY_EMAIL: &str = "dummy@testing.com";
const DUMMY_PWD: &str = "0C4fe7eBbfDbcCBE52DC7A0DdF43bCaeEBaC0EE37bF03C4BAa0ed31eAA03d833";

fn login_dummy_user() -> Result<SessionResponse, Box<dyn Error>> {
   let tx_login = TxLogin::new("", DUMMY_NAME, DUMMY_PWD);
   match tx_login.execute() {
      Err(cause) => {
         let msg = cause.get_msg();
         println!("{:?}", msg);
         Err(msg.into())
      }

      Ok(sess) => Ok(sess)
   }
}

fn signup_dummy_user() -> Result<SessionResponse, Box<dyn Error>> {
   let tx_dummy = TxSignup::new(
      DUMMY_NAME,
      DUMMY_EMAIL,
      DUMMY_PWD);

   match tx_dummy.execute() {
      Err(cause) => {
         let msg = cause.get_msg();
         println!("{:?}", msg);
         Err(msg.into())
      }

      Ok(sess) => Ok(sess)
   }
}

pub fn dummy_setup() -> Result<(), Box<dyn Error>> {
   signup_dummy_user();
   let sess1 = login_dummy_user()?;
   let instance = session::get_instance();
   let sid = sess1.cookie.as_ref();
   instance.get_session_by_cookie(sid)?;
   Ok(())
}