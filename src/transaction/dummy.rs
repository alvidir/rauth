use crate::transaction::signup::TxSignup;

pub fn dummy_setup() {
    let mut tx_dummy = TxSignup::new(
       "dummy".to_string(),
       "dummy@testing.com".to_string(),
       "dummypwd".to_string());
 
    tx_dummy.execute();
 }