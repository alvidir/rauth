//! A bunch of regex definitions and utilities.

use crate::result::{Error, Result};
use regex::Regex;

// include '+' into the charset before '@' in order to allow sufixed emails
pub const EMAIL: &str = r"^[a-zA-Z0-9._-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,63}$";
pub const BASE64: &str = r"^[A-Fa-f0-9]{8,64}$";

/// Returns ok if, and only if, the given string s matches the provided regex.
pub fn match_regex(r: &str, s: &str) -> Result<()> {
    let regex = Regex::new(r).map_err(|err| {
        error!("{} building regex: {:?}", Error::Unknown, err);
        Error::Unknown
    })?;

    if !regex.is_match(s) {
        return Err(Error::RegexNotMatch);
    }

    Ok(())
}
