use std::error::Error;
use crate::transactions::signup::TxSignup;

pub fn dummy_setup() -> Result<(), Box<dyn Error>> {
    let mut tx_dummy = TxSignup::new(
       "dummy",
       "dummy@testing.com",
       "dummypwd");
 
    tx_dummy.execute()?;
    Ok(())
 }