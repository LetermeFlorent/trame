use trame::preferences::vault::{self, Vault};

use super::host;

#[test]
fn each_connection_has_its_own_key() {
    assert_eq!(vault::SSH.key(&host("me@box", None)), "trame/ssh/me@box");
    assert_eq!(
        vault::SSH.key(&host("me@box", Some(2222))),
        "trame/ssh/me@box:2222"
    );
}

#[cfg(windows)]
#[test]
fn a_stored_secret_comes_back_until_it_is_forgotten() {
    let vault = Vault::with_prefix("trame-test/vault/");
    let connection = host(&format!("me@box-{}", std::process::id()), None);
    vault
        .store(&connection, "hunter2 é")
        .expect("the secret is stored");
    assert_eq!(vault.load(&connection).as_deref(), Some("hunter2 é"));
    vault
        .store(&connection, "changed")
        .expect("the secret is replaced");
    assert_eq!(vault.load(&connection).as_deref(), Some("changed"));
    vault.forget(&connection);
    assert_eq!(vault.load(&connection), None);
    vault.forget(&connection);
}

#[cfg(not(windows))]
#[test]
fn without_a_credential_store_nothing_is_kept() {
    let vault = Vault::with_prefix("trame-test/vault/");
    let connection = host("me@box", None);
    assert!(vault.store(&connection, "hunter2").is_err());
    assert_eq!(vault.load(&connection), None);
}
