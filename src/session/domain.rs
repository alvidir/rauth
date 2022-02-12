use std::time::{SystemTime, Duration};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use rand::Rng;
use crate::security::WithOwnedId;
use crate::time;

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct SessionToken {
    pub jti: String,        // JWT ID
    pub exp: usize,         // expiration time (as UTC timestamp) - required
    pub nbf: usize,         // not before time (as UTC timestamp) - non required
    pub iat: SystemTime,    // issued at: creation time
    pub iss: String,        // issuer
    pub sub: String,        // subject
}

impl SessionToken {
    pub fn new(iss: &str, sub: &str, timeout: Duration) -> Self {
        let mut token = SessionToken {
            jti: rand::thread_rng().gen::<u64>().to_string(), // noise
            exp: time::unix_timestamp(SystemTime::now() + timeout),
            nbf: time::unix_timestamp(SystemTime::now()),
            iat: SystemTime::now(),
            iss: iss.to_string(),
            sub: sub.to_string(),
        };

        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        token.jti = hasher.finish().to_string();
        token
    }
}

impl WithOwnedId for SessionToken {
    fn get_id(&self) -> String {
        self.jti.clone()
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct VerificationToken {
    pub jti: String,            
    pub exp: usize,          // expiration time (as UTC timestamp) - required
    pub nbf: usize,          // not before time (as UTC timestamp) - non required
    pub iat: SystemTime,     // issued at: creation time
    pub iss: String,         // issuer
    pub sub: String,
    pub pwd: String,
}

impl VerificationToken {
    pub fn new(iss: &str, email: &str, pwd: &str, timeout: Duration) -> Self {
        let mut token = VerificationToken {
            jti: rand::thread_rng().gen::<u64>().to_string(), // noise
            exp: time::unix_timestamp(SystemTime::now() + timeout),
            nbf: time::unix_timestamp(SystemTime::now()),
            iat: SystemTime::now(),
            iss: iss.to_string(),
            sub: email.to_string(),
            pwd: pwd.to_string(),
        };

        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        token.jti = hasher.finish().to_string();
        token
    }
}


impl WithOwnedId for VerificationToken {
    fn get_id(&self) -> String {
        self.jti.to_string()
    }
}

#[cfg(test)]
pub mod tests {
    use std::time::{SystemTime, Duration};
    use crate::time::unix_timestamp;
    use crate::{security, time};
    use super::{SessionToken, VerificationToken};

    pub const TEST_DEFAULT_TOKEN_TIMEOUT: u64 = 60;
    const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";

    pub fn new_session_token() -> SessionToken {
        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
        SessionToken::new(ISS, &SUB.to_string(), timeout)
    }

    pub fn new_verification_token() -> VerificationToken {
        const ISS: &str = "test";
        const EMAIL: &str = "test@dummy.com ";
        const PWD: &str = "ABCabc123";

        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
        VerificationToken::new(ISS, EMAIL, PWD, timeout)
    }

    #[test]
    fn token_should_not_fail() {
        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);

        let before = SystemTime::now();
        let claim = SessionToken::new(ISS, &SUB.to_string(), timeout);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);     
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));       
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB.to_string(), claim.sub);
    }

    #[test]
    fn token_encode_should_not_fail() {
        const ISS: &str = "test";
        const SUB: i32 = 999;
        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);

        let before = SystemTime::now();
        let claim = SessionToken::new(ISS, &SUB.to_string(), timeout);
        let after = SystemTime::now();
        
        let secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&secret, claim).unwrap();

        let public = base64::decode(JWT_PUBLIC).unwrap();
        let claim = security::verify_jwt::<SessionToken>(&public, &token).unwrap();

        assert!(claim.iat >= before && claim.iat <= after);     
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));       
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB.to_string(), claim.sub);
    }

    #[test]
    fn token_expired_should_fail() {
        use crate::security;
        
        let mut claim = new_session_token();
        claim.exp = time::unix_timestamp(SystemTime::now() - Duration::from_secs(61));
        
        let secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&secret, claim).unwrap();
        let public = base64::decode(JWT_PUBLIC).unwrap();

        assert!(security::verify_jwt::<SessionToken>(&public, &token).is_err());
    }
}
