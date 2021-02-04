extern crate rand;
extern crate ed25519_dalek;
extern crate ecies_ed25519;
use rand::rngs::OsRng;

#[test]
fn test_sign_ed25519() {
    use ed25519_dalek::{
        Signer,
        Verifier,
    };

    let mut csprng = OsRng{};
    let keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut csprng);

    let message: &[u8] = b"Hello world!";
    let signature: ed25519_dalek::Signature = keypair.sign(message);

    assert!(keypair.verify(message, &signature).is_ok());

    let public_key: ed25519_dalek::PublicKey = keypair.public;
    assert!(public_key.verify(message, &signature).is_ok());
}

#[test]
fn test_sign_openssl() {
    use openssl::sign::{Signer, Verifier};
    use openssl::ec::{EcKey,EcGroup, EcPoint};
    use openssl::nid::Nid;
    use openssl::pkey::PKey;
    use openssl::hash::MessageDigest;

    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = EcKey::generate(&group).unwrap();
    let mut ctx = openssl::bn::BigNumContext::new().unwrap();
    
    println!("private eckey = {:?}", key.private_key());

    let bytes = key.public_key().to_bytes(&group,
        openssl::ec::PointConversionForm::COMPRESSED, &mut ctx).unwrap();
    
    println!("public key = {:?}", bytes);

    let public_key = EcPoint::from_bytes(&group, &bytes, &mut ctx).unwrap();
    let ec_key = EcKey::from_public_key(&group, &public_key).unwrap();

    assert!(ec_key.check_key().is_ok());

    let message: &[u8] = b"Hello world!";
    // Sign the data
    let keypair = PKey::from_ec_key(key).unwrap();
    let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
    signer.update(message).unwrap();
    let signature = signer.sign_to_vec().unwrap();

    // Verify the data
    let mut verifier = Verifier::new(MessageDigest::sha256(), &keypair).unwrap();
    assert!(verifier.verify_oneshot(&signature, message).unwrap());
}

#[test]
fn test_parse_openssl() {
    use openssl::sign::{Signer, Verifier};
    use openssl::ec::{EcKey,EcGroup, EcPoint};
    use openssl::nid::Nid;
    use openssl::pkey::PKey;
    use openssl::hash::MessageDigest;
    use openssl::symm::Cipher;

    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = EcKey::generate(&group).unwrap();
    
    let password = b"secret_pwd";
    let pem = key.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), password).unwrap();
    println!("PEM = {:X?}", pem);
    
    let wrong_pwd = b"wrong_pwd";
    assert!(EcKey::private_key_from_pem_passphrase(&pem, wrong_pwd).is_err());
    assert!(EcKey::private_key_from_pem_passphrase(&pem, password).is_ok());

    let mut ctx = openssl::bn::BigNumContext::new().unwrap();
    let bytes = key.public_key().to_bytes(&group, openssl::ec::PointConversionForm::COMPRESSED, &mut ctx).unwrap();
    let point = EcPoint::from_bytes(&group, &bytes, &mut ctx).unwrap();
    EcKey::from_public_key(&group, &point).unwrap();
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
    let code = serde_json::to_string_pretty(&erased).unwrap();

    // Deserialize box back.
    let restored: ErasedPwBox = serde_json::from_str(&code).unwrap();
    let plaintext = eraser.restore(&restored).unwrap().open(pwd).unwrap();
    assert_eq!(&*plaintext, data);
}