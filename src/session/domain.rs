use std::time::{SystemTime, Duration};
use std::hash::Hash;
use crate::time::unix_timestamp;

#[derive(Serialize, Deserialize, Hash)]
pub struct SessionToken {
    pub exp: usize,          // expiration time (as UTC timestamp) - required
    pub iat: SystemTime,     // issued at: creation time
    pub iss: String,         // issuer
    pub sub: i32,
}

impl SessionToken {
    pub fn new(iss: &str, sub: i32, timeout: Duration) -> Self {
        SessionToken {
            exp: unix_timestamp(SystemTime::now() + timeout),
            iat: SystemTime::now(),
            iss: iss.to_string(),
            sub: sub,
        }
    }
}

#[derive(Serialize, Deserialize, Hash)]
pub struct VerificationToken {
    pub exp: usize,          // expiration time (as UTC timestamp) - required
    pub iat: SystemTime,     // issued at: creation time
    pub iss: String,         // issuer
    pub sub: Option<String>,
    pub pwd: Option<String>,
}

impl VerificationToken {
    pub fn new(iss: &str, email: Option<String>, pwd: Option<String>, timeout: Duration) -> Self {
        VerificationToken {
            exp: unix_timestamp(SystemTime::now() + timeout),
            iat: SystemTime::now(),
            iss: iss.to_string(),
            sub: email.clone(),
            pwd: pwd.clone(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::time::{SystemTime, Duration};
    use crate::time::unix_timestamp;
    use crate::security;
    use super::SessionToken;

    const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";

    #[test]
    fn token_should_not_fail() {
        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = SessionToken::new(ISS, SUB, timeout);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);     
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));       
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB, claim.sub);
    }

    #[test]
    fn token_encode_should_not_fail() {
        const ISS: &str = "test";
        const SUB: i32 = 999;
        let timeout = Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = SessionToken::new(ISS, SUB, timeout);
        let after = SystemTime::now();
        
        let secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&secret, claim).unwrap();

        let public = base64::decode(JWT_PUBLIC).unwrap();
        let claim = security::verify_jwt::<SessionToken>(&public, &token).unwrap();

        assert!(claim.iat >= before && claim.iat <= after);     
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));       
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB, claim.sub);
    }

    #[test]
    fn token_expired_should_fail() {
        use crate::security;

        const ISS: &str = "test";
        const SUB: i32 = 999;
        const IT_LIMIT: i32 = 100_000;
        let timeout = Duration::from_secs(0);

        let claim = SessionToken::new(ISS, SUB, timeout);
        let secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&secret, claim).unwrap();

        let public = base64::decode(JWT_PUBLIC).unwrap();

        let mut iterations: i32 = 0;
        while iterations < IT_LIMIT && security::verify_jwt::<SessionToken>(&public, &token).is_ok() {
            iterations += 1;
        }


        assert!(security::verify_jwt::<SessionToken>(&public, &token).is_err());
    }
}
