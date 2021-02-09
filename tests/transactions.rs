use std::error::Error;
extern crate tp_auth;

pub use tp_auth::transactions;

const DUMMY_NAME: &str = "dummy";
const DUMMY_EMAIL: &str = "dummy@testing.com";
const DUMMY_PWD: &str = "0C4fe7eBbfDbcCBE52DC7A0DdF43bCaeEBaC0EE37bF03C4BAa0ed31eAA03d833";

#[test]
fn signup() -> Result<(), Box<dyn Error>> {
    use super::transactions;

    let tx_dummy = transactions::TxSignup::new(
       DUMMY_NAME,
       DUMMY_EMAIL,
       DUMMY_PWD);
 
    tx_dummy.execute()
 }