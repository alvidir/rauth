use std::hash::Hasher;
use std::hash::Hash;
use std::fmt;
use rand::Rng;
use rand::prelude::ThreadRng;
use std::time::SystemTime;
//use crypto::digest::Digest;
//use crypto::sha2::Sha256;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~?][+-";

//#[derive(PartialEq, Eq)]
//#[derive(Hash)]
#[derive(Clone)]
pub struct Token (String, SystemTime);

impl Token {
    pub fn new(rand: &mut ThreadRng, date: SystemTime, size: usize) -> Self {
        let value: String = (0..size)
        .map(|_| {
            let idx = rand.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
        Token(value, date)
    }

    pub fn from_string(tid: &str) -> Token {
        Token(tid.to_string(),  SystemTime::now())
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn is_alive(&self) -> bool {
        self.1 < SystemTime::now()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Hash for Token {
    fn hash<H>(&self, h: &mut H)
    where
        H: Hasher,
    {
        self.0.hash(h)
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Token {}