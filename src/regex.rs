use regex::Regex;
use std::error::Error;
use crate::constants;

const REGEX_NAME: &str = r"^[-_A-Za-z0-9\.]+$";
const REGEX_EMAIL: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,63}$";
const REGEX_HASH: &str = r"\b[A-Fa-f0-9]{8, 64}\b";
const REGEX_B64: &str = r"^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$";
const REGEX_URL: &str = r#"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}/?$"#;
const REGEX_COOKIE: &str = r"^[A-Za-z0-9)(*&^%$#@!~?\]\[+-]+$";

const ERR_REGEX_NOT_MATCH: &str = "regex does not match";

pub fn match_name(name: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_NAME).unwrap();
    if !regex.is_match(name) {
        return Err(ERR_REGEX_NOT_MATCH.into());
    }

    Ok(())
}

pub fn match_email(email: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_EMAIL).unwrap();
    if !regex.is_match(email) {
        return Err(ERR_REGEX_NOT_MATCH.into());
    }

    Ok(())
}

pub fn match_pwd(pwd: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_HASH).unwrap();
    if !regex.is_match(pwd) {
        return Err(ERR_REGEX_NOT_MATCH.into());
    }

    Ok(())
}

pub fn match_base64(data: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_B64).unwrap();
    if !regex.is_match(data) {
        return Err(ERR_REGEX_NOT_MATCH.into());
    }

    Ok(())
}

pub fn match_url(data: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_URL).unwrap();
    if !regex.is_match(data) {
        return Err(ERR_REGEX_NOT_MATCH.into());
    }

    Ok(())
}

pub fn match_cookie(cookie: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_COOKIE)?;
    if !regex.is_match(cookie) {
        return Err(ERR_REGEX_NOT_MATCH.into());
    }

    Ok(())
}