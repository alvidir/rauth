use crate::transaction::traits::*;
use crate::transaction::factory::*;
use crate::service::session::transactions::login::TxLogin;

pub fn new_tx_login() -> Box<dyn Tx> {
    new_transaction(&TxLogin{})
}