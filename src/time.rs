use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn _unix_seconds(current: SystemTime) -> Result<u64, Box<dyn Error>> {
	match current.duration_since(UNIX_EPOCH) {
		Err(err) => {
			let msg = format!("Time went backwards: {}", err);
			return Err(msg.into());
		}

		Ok(unix) => Ok(unix.as_secs())
	}
}