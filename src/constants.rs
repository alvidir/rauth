pub const TOTP_SECRET_LEN: usize = 32_usize;
pub const TOTP_SECRET_NAME: &str = ".totp_secret";
pub const TOKEN_ISSUER: &str = "rauth.alvidir.com";
pub const PWD_SUFIX: &str = "::PWD::RAUTH";

pub const ERR_NOT_FOUND: &str = "E-001";
pub const ERR_UNAUTHORIZED: &str = "E-002";
pub const ERR_PARSE_HEADER: &str = "E-003";
pub const ERR_HEADER_REQUIRED: &str = "E-004";
pub const ERR_SIGN_TOKEN: &str = "E-005";
pub const ERR_VERIFY_TOKEN: &str = "E-006";
pub const ERR_DECRYPT_TOKEN: &str = "E-007";
pub const ERR_UNVERIFIED: &str = "E-008";
pub const ERR_INVALID_OPTION: &str = "E-009";
pub const ERR_SEND_EMAIL: &str = "E-010";