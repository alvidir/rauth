#![allow(unused)]
use regex::Regex;
use std::error::Error;

const REGEX_NAME: &str = r"^[-_A-Za-z0-9\.]+$";
const REGEX_EMAIL: &str = r"^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,63}$";
const REGEX_HASH256: &str = r"\b[A-Fa-f0-9]{64}\b";
const REGEX_B64: &str = r"^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$";
const REGEX_URL: &str = r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)";
const REGEX_COOKIE: &str = r"^[A-Za-z0-9)(*&^%$#@!~?\]\[+-]+$";
const COOKIE_SEPARATOR: &str = "=";

const ERR_NAME_FORMAT: &str = "The Name only allows alphanumeric characters";
const ERR_EMAIL_FORMAT: &str = "The provided email does not match with any real address";
const ERR_PWD_FORMAT: &str = "The password must contains, at least, an upper and lower case letters, as well as some numbers and special characters";
const ERR_DATA_FORMAT: &str = "The provided data does not match with base 64 format";
const ERR_URL_FORMAT: &str = "The provided string does not match with the url standard";
const ERR_COOKIE_FORMAT: &str = "The provided cookie does not match with a real one";

pub fn match_name(name: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_NAME).unwrap();
    if !regex.is_match(name) {
        return Err(ERR_NAME_FORMAT.into());
    }

    Ok(())
}

pub fn match_email(email: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_EMAIL).unwrap();
    if !regex.is_match(email) {
        return Err(ERR_EMAIL_FORMAT.into());
    }

    Ok(())
}

pub fn match_pwd(pwd: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_HASH256).unwrap();
    if !regex.is_match(pwd) {
        return Err(ERR_PWD_FORMAT.into());
    }

    Ok(())
}

pub fn match_base64(data: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_B64).unwrap();
    if !regex.is_match(data) {
        return Err(ERR_DATA_FORMAT.into());
    }

    Ok(())
}

pub fn match_url(data: &str) -> Result<(), Box<dyn Error>> {
    let regex = Regex::new(REGEX_URL).unwrap();
    if !regex.is_match(data) {
        return Err(ERR_URL_FORMAT.into());
    }

    Ok(())
}

pub fn match_cookie(cookie: &str) -> Result<(), Box<dyn Error>> {
    let split: Vec<&str> = cookie.split(COOKIE_SEPARATOR).collect();
    if split.len() != 2 {
        return Err(ERR_COOKIE_FORMAT.into());
    }

    let regex = Regex::new(REGEX_COOKIE).unwrap();
    if !regex.is_match(split[0]) {
        return Err(ERR_COOKIE_FORMAT.into());
    }

    match_email(split[1])
}