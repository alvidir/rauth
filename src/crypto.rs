use std::error::Error;
use rand::rngs::OsRng;
use std::str::from_utf8;
use pwbox::{Eraser, ErasedPwBox, Suite, sodium::Sodium};
use ed25519_dalek::{
    Keypair,
    Signature,
    Signer,
    PublicKey,
    SecretKey,
    Verifier,
    KEYPAIR_LENGTH,
    SECRET_KEY_LENGTH,
    PUBLIC_KEY_LENGTH,
};

pub fn new_ed25519() -> Keypair {
    let mut csprng = OsRng{};
    Keypair::generate(&mut csprng)
}

pub fn encrypt(pwd: &str, data: &[u8]) -> Result<String, Box<dyn Error>> {
    use rand_core::OsRng;

    let mut csprng = OsRng{};
    let pwbox = Sodium::build_box(&mut csprng)
                .seal(pwd, data)?;
    
    let mut eraser = Eraser::new();
    eraser.add_suite::<Sodium>();
    let erased: ErasedPwBox = eraser.erase(&pwbox)?;
    let serial = serde_json::to_string_pretty(&erased)?;
    let code = base64::encode(&serial);
    Ok(code)
}

pub fn decrypt<'a>(pwd: &str, data: &[u8]) -> Result<pwbox::SensitiveData, Box<dyn Error>> {
    let mut eraser = Eraser::new();
    eraser.add_suite::<Sodium>();

    let serial = base64::decode(&data)?;
    let parsed = from_utf8(&serial)?;
    let erased: ErasedPwBox = serde_json::from_str(parsed)?;
    let restored = eraser.restore(&erased)?.open(pwd)?;
    Ok(restored)
}