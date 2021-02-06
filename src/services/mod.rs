pub mod session;

use std::error::Error;
use tonic::{Status, Code};

pub fn parse_error(err: Box<dyn Error>) -> Status {
    println!("{:?}", err.to_string());
    let code = Code::from(0);
    Status::new(code, err.to_string())
}