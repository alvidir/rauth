pub const TOTP_SECRET_LEN: usize = 32_usize;
pub const TOTP_SECRET_NAME: &str = ".totp_secret";
pub const TOKEN_ISSUER: &str = "rauth.alvidir.com";
pub const PWD_SUFIX: &str = "::PWD::RAUTH";

pub const ERR_NOT_FOUND: &str = "E-001";
pub const ERR_UNAUTHORIZED: &str = "E-002";
pub const ERR_SIGN_TOKEN: &str = "E-003"; // cannot sign token
pub const ERR_PARSE_TOKEN: &str = "E-004"; // cannot parse token
pub const ERR_TOKEN_REQUIRED: &str = "E-005"; // token required
pub const ERR_VERIFY_TOKEN: &str = "E-006"; // cannot verify token
pub const ERR_DECRYPT_TOKEN: &str = "E-007"; // cannot descript token
pub const ERR_UNVERIFIED: &str = "E-008"; // unverified signup is not allowed
pub const ERR_MISSING_DATA: &str = "E-009"; // some data is missing
pub const ERR_INVALID_OPTION: &str = "E-010"; // invalid option