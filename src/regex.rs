//! A bunch of regex definitions and utilities.

use crate::result::{Error, Result};
use regex::Regex;

/// Returns ok if, and only if, the given string s matches the provided regex.
pub fn match_regex(r: &str, s: &str) -> Result<()> {
    let regex = Regex::new(r).map_err(|err| {
        error!(error = err.to_string(), "building regex");
        Error::Unknown
    })?;

    if !regex.is_match(s) {
        return Err(Error::RegexNotMatch);
    }

    Ok(())
}
