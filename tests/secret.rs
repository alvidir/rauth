extern crate rand;
extern crate ed25519_dalek;
extern crate ecies_ed25519;

use rand::rngs::OsRng;
use ed25519_dalek::{
    Keypair,
    Signature,
    Signer,
    PublicKey,
    Verifier
};

#[test]
fn test_sign_msg() {
    let mut csprng = OsRng{};
    let keypair: Keypair = Keypair::generate(&mut csprng);

    let message: &[u8] = b"Hello world!";
    let signature: Signature = keypair.sign(message);

    assert!(keypair.verify(message, &signature).is_ok());

    let public_key: PublicKey = keypair.public;
    assert!(public_key.verify(message, &signature).is_ok());
}

#[test]
fn test_encrypt() {
    let mut csprng = OsRng{};
    let (secret, public) = ecies_ed25519::generate_keypair(&mut csprng);
    let message: &[u8] = b"Hello world!";
    // Encrypt the message with the public key such that only the holder of the secret key can decrypt.
    let encrypted = ecies_ed25519::encrypt(&public, message, &mut csprng).unwrap();
    // Decrypt the message with the secret key
    let decrypted = ecies_ed25519::decrypt(&secret, &encrypted).unwrap();
    assert_eq!(message, decrypted);
}

#[test]
fn test_password() {
    use pwbox::{Eraser, ErasedPwBox, Suite, sodium::Sodium};
    use rand_core::OsRng;

    let pwd = b"password";
    let data =  b"Hello world!";

    // Create a new box.
    let mut csprng = OsRng{};
    let pwbox = Sodium::build_box(&mut csprng)
                .seal(pwd, data).unwrap();

    // Serialize box.
    let mut eraser = Eraser::new();
    eraser.add_suite::<Sodium>();
    let erased: ErasedPwBox = eraser.erase(&pwbox).unwrap();
    println!("{}", serde_json::to_string_pretty(&erased).unwrap());
    
    // Deserialize box back.
    let plaintext = eraser.restore(&erased).unwrap().open(pwd).unwrap();
    assert_eq!(&*plaintext, data);
}