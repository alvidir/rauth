pub mod session;

use tonic::{Status, Code};
use crate::transactions::Cause;

pub fn parse_cause(cause: Box<dyn Cause>) -> Status {
    let code = Code::from(cause.get_code());
    let cause = cause.get_msg();
    Status::new(code, cause)
}