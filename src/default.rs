pub const SERVER_IP: &str = "127.0.0.1";

pub const TOKEN_LEN: usize = 8;
pub const TOKEN_TIMEOUT: u64 = 86400; // 3600s * 24h

pub const CONNECTION_TIMEOUT: u64 = 100; // in seconds
pub const CONNECTION_SLEEP: u64 = 1; // in seconds

pub const RSA_NAME: &str = "default_rsa.pem";

pub const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
pub const ENV_POSTGRES_DSN: &str = "DATABASE_URL";
pub const ENV_MONGO_DSN: &str = "MONGO_DSN";
pub const ENV_MONGO_DB: &str = "MONGO_DB";
pub const ENV_MONGO_COLL: &str = "MONGO_COLLECTION";

#[cfg(test)]
pub mod tests {
    pub static DUMMY_NAME: &str = "dummy";
    pub static DUMMY_EMAIL: &str = "dummy@testing.com";
    pub static DUMMY_PWD: &str = "0C4fe7eBbfDbcCBE";
    pub static DUMMY_URL: &str = "dummy.com";
    pub static DUMMY_DESCR: &str = "this is a dummy application";

    pub fn get_prefixed_data(subject: &str, is_app: bool) -> (String, String) {
        let name = {
            if is_app {
                format!("{}_{}_app", subject, DUMMY_NAME)
            } else {
                format!("{}_{}_user", subject, DUMMY_NAME)
            }
        };

        let email = {
            if is_app {
                format!("http://{}.{}", subject, DUMMY_URL)
            } else {
                format!("{}_{}", subject, DUMMY_EMAIL)
            }
        };

        (name, email)
    }
}