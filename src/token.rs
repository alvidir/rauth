use std::time::{SystemTime, Duration};
use crate::time::unix_timestamp;

// token for email-verification
#[derive(Serialize, Deserialize)]
pub struct Token {
    pub(super) exp: usize,          // expiration time (as UTC timestamp) - required
    pub(super) iat: SystemTime,     // issued at: creation time
    pub(super) iss: String,         // issuer
    pub(super) sub: i32,            // subject: the user id
}

impl Token {
    pub fn new(iss: &str, sub: i32, timeout: Duration) -> Self {
        Token {
            exp: unix_timestamp(SystemTime::now() + timeout),
            iat: SystemTime::now(),
            iss: iss.to_string(),
            sub: sub,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::time::{SystemTime, Duration};
    use crate::time::unix_timestamp;
    use crate::security;
    use super::Token;

    const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";

    #[test]
    fn user_token_should_not_fail() {
        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(ISS, SUB, timeout);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);     
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));       
        assert_eq!("rauth.alvidir.com", claim.iss);
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB, claim.sub);
    }

    #[test]
    fn user_token_encode_should_not_fail() {
        const ISS: &str = "test";
        const SUB: i32 = 999;
        let timeout = Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(ISS, SUB, timeout);
        let after = SystemTime::now();
        
        let token = security::encode_jwt(JWT_SECRET, claim).unwrap();
        let claim = security::decode_jwt::<Token>(JWT_PUBLIC, &token).unwrap();

        assert!(claim.iat >= before && claim.iat <= after);     
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));       
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB, claim.sub);
    }

    #[test]
    fn user_token_expired_should_fail() {
        use std::thread::sleep;
        use crate::security;

        dotenv::dotenv().unwrap();

        const ISS: &str = "test";
        const SUB: i32 = 999;
        let timeout = Duration::from_secs(0);

        let claim = Token::new(ISS, SUB, timeout);
        let token = security::encode_jwt(JWT_SECRET, claim).unwrap();
        
        sleep(Duration::from_secs(1));
        assert!(security::decode_jwt::<Token>(JWT_PUBLIC, &token).is_err());
    }
}