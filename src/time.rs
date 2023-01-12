//! Datetime related utilities.

use chrono::prelude::{DateTime, Utc};
use std::time::SystemTime;

/// Given a system time returns its corresponding unix timestamp
pub fn unix_timestamp(current: SystemTime) -> usize {
    let utc: DateTime<Utc> = current.into();
    utc.timestamp() as usize
    // formats like "2001-07-08T00:34:60.026490+09:30"
    // see: https://stackoverflow.com/questions/64146345/how-do-i-convert-a-systemtime-to-iso-8601-in-rust
}
