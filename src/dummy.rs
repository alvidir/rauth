use std::error::Error;
use crate::transactions::signup::TxSignup;
use crate::transactions::login::TxLogin;

const DUMMY_NAME: &str = "dummy";
const DUMMY_EMAIL: &str = "dummy@testing.com";
const DUMMY_PWD: &str = "0C4fe7eBbfDbcCBE52DC7A0DdF43bCaeEBaC0EE37bF03C4BAa0ed31eAA03d833";

//const INFO_SIGNUP: &str = "Signing up the dummy user";
//const INFO_LOGIN: &str = "Loging in the dummy user";
//const INFO_COOKIE: &str = "The dummy's session got cookie";

fn login_dummy_user() -> Result<(), Box<dyn Error>> {
   let tx_login = TxLogin::new("", DUMMY_EMAIL, DUMMY_PWD);
   tx_login.execute()?;
   Ok(())
}

fn signup_dummy_user() -> Result<(), Box<dyn Error>> {
   let tx_dummy = TxSignup::new(
      DUMMY_NAME,
      DUMMY_EMAIL,
      DUMMY_PWD);

   tx_dummy.execute()?;
   login_dummy_user()
}

pub fn dummy_setup() -> Result<(), Box<dyn Error>> {
   if login_dummy_user().is_err() {
      return signup_dummy_user();
   }

   Ok(())
}