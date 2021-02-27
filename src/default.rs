pub const SERVER_IP: &str = "127.0.0.1";

pub const TOKEN_LEN: usize = 8;
pub const TOKEN_TIMEOUT: u64 = 86400; // 3600s * 24h

pub const RSA_NAME: &str = "default_rsa.pem";

pub const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
pub const ENV_POSTGRES_DSN: &str = "DATABASE_URL";
pub const ENV_MONGO_DSN: &str = "MONGO_DSN";
pub const ENV_MONGO_DB: &str = "MONGO_DB";
pub const ENV_MONGO_COLL: &str = "MONGO_COLLECTION";