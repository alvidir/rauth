pub const SERVER_IP: &str = "127.0.0.1";

pub const TOKEN_LEN: usize = 32;
pub const TOKEN_TIMEOUT: u64 = 86400; // 3600s * 24h

pub const CONNECTION_TIMEOUT: u64 = 100; // in seconds
pub const CONNECTION_SLEEP: u64 = 1; // in seconds

pub const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
pub const ENV_POSTGRES_DSN: &str = "DATABASE_URL";
pub const ENV_MONGO_DSN: &str = "MONGO_DSN";
pub const ENV_MONGO_DB: &str = "MONGO_DB";