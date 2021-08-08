use regex::Regex;
use std::error::Error;

pub const EMAIL: &str = r"^[a-zA-Z0-9._-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,63}$"; // include '+' into charset before '@' in order to allow sufixed emails
pub const BASE64: &str = r"\b[A-Fa-f0-9]{8, 64}\b";
pub const URL: &str = r#"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}/?$"#;

const ERR_REGEX_NOT_MATCH: &str = "regex does not match";

pub fn match_regex(r: &str, s: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(r)?;
    if !regex.is_match(s) {
        return Err(ERR_REGEX_NOT_MATCH.into());
    }

    Ok(())
}