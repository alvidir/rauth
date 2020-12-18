use crate::transactions::signup::TxSignup;

pub fn dummy_setup() -> Result<(), String> {
    let mut tx_dummy = TxSignup::new(
       "dummy".to_string(),
       "dummy@testing.com".to_string(),
       "dummypwd".to_string());
 
    tx_dummy.execute();
    match tx_dummy.result() {
       None => {
         Ok(())
       }

       Some(result) => {
         result?;
         Ok(())
       }
    }
 }