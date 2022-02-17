use regex::Regex;
use std::error::Error;
use crate::constants;

// include '+' into charset before '@' in order to allow sufixed emails
pub const EMAIL: &str = r"^[a-zA-Z0-9._-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,63}$";
pub const BASE64: &str = r"^[A-Fa-f0-9]{8,64}$";

pub fn match_regex(r: &str, s: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(r)?;
    if !regex.is_match(s) {
        return Err(constants::ERR_REGEX_NOT_MATCH.into());
    }

    Ok(())
}