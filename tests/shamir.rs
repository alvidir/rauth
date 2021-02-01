use shamir::SecretData;

#[test]
fn test_split() {
    let msg = "Hello world!";
    let needed = 3;

    let secret_data = SecretData::with_secret(msg, needed);
    println!("{:?}", secret_data.secret_data);

    let share1 = secret_data.get_share(1);
    let share2 = secret_data.get_share(2);
    let share3 = secret_data.get_share(3);
    let share4 = secret_data.get_share(4);

    let mut recovered = SecretData::recover_secret(3, vec![share1, share2, share3]).unwrap();
    assert_eq!(recovered, msg);

    let share5 = secret_data.get_share(5);
    let share6 = secret_data.get_share(6);

    recovered = SecretData::recover_secret(3, vec![share4, share5, share6]).unwrap();
    assert_eq!(recovered, msg);
}