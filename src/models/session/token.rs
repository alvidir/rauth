use std::fmt;
use rand::Rng;
use rand::prelude::ThreadRng;
//use crypto::digest::Digest;
//use crypto::sha2::Sha256;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~?][+-\
                        _";

#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub struct Token (String);

impl Token {
    pub fn new(rand: &mut ThreadRng, size: usize) -> Self {
        let value: String = (0..size)
        .map(|_| {
            let idx = rand.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
        Token(value)
    }

    //pub fn reset(&mut self, rand: &mut ThreadRng) {
    //    let value: String = (0..self.0.len())
    //    .map(|_| {
    //        let idx = rand.gen_range(0, CHARSET.len());
    //        CHARSET[idx] as char
    //    })
    //    .collect();
    //
    //    self.0 = value;
    //}

    //pub fn digest(&self) -> String {
    //    let mut hasher = Sha256::new();
    //    hasher.input_str(&self.0);
    //    hasher.result_str()
    //}

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}