use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::prelude::{DateTime, Utc};

pub fn _unix_seconds(current: SystemTime) -> Result<u64, Box<dyn Error>> {
	match current.duration_since(UNIX_EPOCH) {
		Err(err) => {
			let msg = format!("Time went backwards: {}", err);
			return Err(msg.into());
		}

		Ok(unix) => Ok(unix.as_secs())
	}
}

pub fn unix_timestamp(current: SystemTime) -> usize {
    let utc: DateTime<Utc> = current.into();
    utc.timestamp() as usize
    // formats like "2001-07-08T00:34:60.026490+09:30"
	// see: https://stackoverflow.com/questions/64146345/how-do-i-convert-a-systemtime-to-iso-8601-in-rust
}

pub fn _iso8601(current: SystemTime) -> String {
    let utc: DateTime<Utc> = current.into();
    format!("{}", utc.format("%+"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}