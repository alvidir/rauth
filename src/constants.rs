pub const SERVER_IP: &str = "127.0.0.1";

pub const TOKEN_LEN: usize = 8;
pub const TOKEN_TIMEOUT: u64 = 86400; // 3600s * 24h

pub const CONNECTION_TIMEOUT: u64 = 10; // in seconds
pub const CONNECTION_SLEEP: u64 = 1; // in seconds

pub const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
pub const ENV_POSTGRES_DSN: &str = "DATABASE_URL";
pub const ENV_MONGO_DSN: &str = "MONGO_DSN";
pub const ENV_MONGO_DB: &str = "MONGO_DB";
pub const ENV_SMTP_TRANSPORT: &str = "SMTP_TRANSPORT";
pub const ENV_SMTP_ORIGIN: &str = "SMTP_ORIGIN";
pub const ENV_SMTP_USERNAME: &str = "SMTP_USERNAME";
pub const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
pub const ENV_SECRET_PWD: &str = "SECRET_PWD";
 
pub const ERR_NOT_FOUND: &str = "not found";
pub const ERR_ALREADY_EXISTS: &str = "already exists";
pub const ERR_POISONED: &str = "the lock or mutex got poisoned due to a thread's panic while holding it";
