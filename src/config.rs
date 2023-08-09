use base64::{engine::general_purpose, Engine as _};
use once_cell::sync::Lazy;
use std::env;

pub const DEFAULT_PORT: &str = "8000";
pub const DEFAULT_ADDR: &str = "127.0.0.1";
pub const DEFAULT_JWT_HEADER: &str = "authorization";
pub const DEFAULT_TOTP_HEADER: &str = "x-totp-secret";
pub const DEFAULT_TOKEN_TIMEOUT: u64 = 7200;
pub const DEFAULT_TOTP_SECRET_LEN: usize = 32_usize;
#[allow(dead_code)]
pub const DEFAULT_POOL_SIZE: u32 = 10;
#[allow(dead_code)]
pub const DEFAULT_CONN_TIMEOUT: u32 = 100; //ms

const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const ENV_SERVICE_ADDR: &str = "SERVICE_ADDR";
const ENV_JWT_SECRET: &str = "JWT_SECRET";
const ENV_JWT_PUBLIC: &str = "JWT_PUBLIC";
const ENV_JWT_HEADER: &str = "JWT_HEADER";
const ENV_TOTP_HEADER: &str = "TOTP_HEADER";
const ENV_TOKEN_TIMEOUT: &str = "TOKEN_TIMEOUT";

const ENV_PWD_SUFIX: &str = "PWD_SUFIX";
const ENV_TOTP_SECRET_LEN: &str = "TOTP_SECRET_LEN";
const ENV_TOKEN_ISSUER: &str = "TOKEN_ISSUER";

pub static SERVICE_ADDR: Lazy<String> = Lazy::new(|| {
    let netw = env::var(ENV_SERVICE_ADDR).unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let port = env::var(ENV_SERVICE_PORT).unwrap_or_else(|_| DEFAULT_PORT.to_string());
    format!("{}:{}", netw, port)
});

pub static TOKEN_TIMEOUT: Lazy<u64> = Lazy::new(|| {
    env::var(ENV_TOKEN_TIMEOUT)
        .map(|timeout| timeout.parse().unwrap())
        .unwrap_or(DEFAULT_TOKEN_TIMEOUT)
});

pub static JWT_SECRET: Lazy<Vec<u8>> = Lazy::new(|| {
    env::var(ENV_JWT_SECRET)
        .map(|secret| general_purpose::STANDARD.decode(secret).unwrap())
        .expect("jwt secret must be set")
});

pub static JWT_PUBLIC: Lazy<Vec<u8>> = Lazy::new(|| {
    env::var(ENV_JWT_PUBLIC)
        .map(|secret| general_purpose::STANDARD.decode(secret).unwrap())
        .expect("jwt public key must be set")
});

pub static JWT_HEADER: Lazy<String> =
    Lazy::new(|| env::var(ENV_JWT_HEADER).unwrap_or_else(|_| DEFAULT_JWT_HEADER.to_string()));

pub static TOTP_HEADER: Lazy<String> =
    Lazy::new(|| env::var(ENV_TOTP_HEADER).unwrap_or_else(|_| DEFAULT_TOTP_HEADER.to_string()));

pub static PWD_SUFIX: Lazy<String> =
    Lazy::new(|| env::var(ENV_PWD_SUFIX).expect("password sufix must be set"));

pub static TOTP_SECRET_LEN: Lazy<usize> = Lazy::new(|| {
    env::var(ENV_TOTP_SECRET_LEN)
        .map(|len| len.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_TOTP_SECRET_LEN)
});

pub static TOKEN_ISSUER: Lazy<String> =
    Lazy::new(|| env::var(ENV_TOKEN_ISSUER).expect("token issuer must be set"));
