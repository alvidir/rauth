pub const TOTP_SECRET_LEN: usize = 32_usize;
pub const TOTP_SECRET_NAME: &str = ".totp_secret";
pub const TOKEN_ISSUER: &str = "rauth.alvidir.com";
pub const EMAIL_VERIFICATION_SUBJECT: &str = "Email verification";
pub const EMAIL_VERIFICATION_TEMPLATE: &str = "verification_email.html";
pub const EMAIL_RESET_PASSWORD_SUBJECT: &str = "Reset password";
pub const EMAIL_RESET_PASSWORD_TEMPLATE: &str = "reset_pwd_email.html";

pub const ERR_NOT_FOUND: &str = "E-001";
pub const ERR_WRONG_CREDENTIALS: &str = "E-002";
pub const ERR_UNAUTHORIZED: &str = "E-003";
pub const ERR_VERIFY_TOKEN: &str = "E-004";
pub const ERR_INVALID_EMAIL_FORMAT: &str = "E-005";
pub const ERR_INVALID_PWD_FORMAT: &str = "E-006";


pub const ERR_PARSE_HEADER: &str = "E-013";
pub const ERR_HEADER_REQUIRED: &str = "E-014";
pub const ERR_MALFORMED_TOKEN: &str = "E-017";
pub const ERR_PARSE_TOKEN: &str = "E-018";
pub const ERR_UNVERIFIED: &str = "E-019";
pub const ERR_INVALID_OPTION: &str = "E-020";
pub const ERR_SEND_EMAIL: &str = "E-021";
pub const ERR_UNKNOWN: &str = "E-999";