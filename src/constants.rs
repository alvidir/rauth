pub mod settings {
    pub const SERVER_IP: &str = "127.0.0.1";
    pub const TOKEN_LEN: usize = 8;
    pub const TOKEN_TIMEOUT: u64 = 86400; // 3600s * 24h
    pub const POOL_SIZE: u32 = 1_u32; // by constants: single thread
}

pub mod environment {
    pub const SERVICE_PORT: &str = "SERVICE_PORT";
    pub const POSTGRES_DSN: &str = "DATABASE_URL";
    pub const MONGO_DSN: &str = "MONGO_DSN";
    pub const MONGO_DB: &str = "MONGO_DB";
    pub const SMTP_TRANSPORT: &str = "SMTP_TRANSPORT";
    pub const SMTP_ORIGIN: &str = "SMTP_ORIGIN";
    pub const SMTP_USERNAME: &str = "SMTP_USERNAME";
    pub const SMTP_PASSWORD: &str = "SMTP_PASSWORD";
    pub const SECRET_PEM: &str = "SECRET_PEM";
    pub const SECRET_PWD: &str = "SECRET_PWD";
    pub const VERIFY_URL: &str = "VERIFY_URL";
    pub const SUPPORT_EMAIL: &str = "SUPPORT_EMAIL";
}

pub mod errors {
    pub const CANNOT_CONNECT: &str = "cannot connect";
    pub const NOT_FOUND: &str = "not found";
    pub const ALREADY_EXISTS: &str = "already exists";
    pub const POISONED: &str = "poisoned resource";
    pub const NOT_VERIFIED: &str = "verification required";
    pub const UNAUTHORIZED: &str = "unauthorized";
    pub const PARSE_FAILED: &str = "could not parse";
}
