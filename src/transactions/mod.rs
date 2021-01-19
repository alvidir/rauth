use std::fmt;
pub mod client;

pub trait Cause {
	fn get_code(&self) -> i32;
	fn get_msg(&self) -> &str;
}

struct TxCause {
	code: i32,
	msg: String,
}

impl TxCause {
	fn new(code: i32, msg: String) -> impl Cause {
		TxCause {
			code: code,
			msg: msg,
		}
	}
}

impl Cause for TxCause {
	fn get_code(&self) -> i32 {
		self.code
	}

	fn get_msg(&self) -> &str {
		&self.msg
	}
}

impl fmt::Display for dyn Cause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_msg())
    }
}