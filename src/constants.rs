pub const TOTP_SECRET_LEN: usize = 32_usize;
pub const TOTP_SECRET_NAME: &str = ".totp_secret";
pub const TOKEN_ISSUER: &str = "rauth.alvidir.com";
pub const EMAIL_VERIFICATION_SUBJECT: &str = "Email verification";
pub const EMAIL_VERIFICATION_TEMPLATE: &str = "verification_email.html";
pub const EMAIL_RESET_SUBJECT: &str = "Reset password";
pub const EMAIL_RESET_TEMPLATE: &str = "reset_email.html";

pub const ERR_UNKNOWN: &str = "E001";
pub const ERR_NOT_FOUND: &str = "E002";
pub const ERR_NOT_AVAILABLE: &str = "E003";
pub const ERR_UNAUTHORIZED: &str = "E004";
pub const ERR_INVALID_TOKEN: &str = "E005";
pub const ERR_INVALID_FORMAT: &str = "E006";
pub const ERR_INVALID_HEADER: &str = "E007";
pub const ERR_WRONG_CREDENTIALS: &str = "E008";
pub const ERR_REGEX_NOT_MATCH: &str = "E009";