use regex::Regex;
use std::error::Error;

pub const _NAME: &str = r"^[-_A-Za-z0-9\.]+$";
pub const EMAIL: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,63}$";
pub const _HASH: &str = r"\b[A-Fa-f0-9]{8, 64}\b";
pub const _BASE64: &str = r"^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$";
pub const URL: &str = r#"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}/?$"#;
pub const _COOKIE: &str = r"^[A-Za-z0-9)(*&^%$#@!~?\]\[+-]+$";

const ERR_REGEX_NOT_MATCH: &str = "regex does not match";

pub fn match_regex(r: &str, s: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(r)?;
    if !regex.is_match(s) {
        return Err(ERR_REGEX_NOT_MATCH.into());
    }

    Ok(())
}