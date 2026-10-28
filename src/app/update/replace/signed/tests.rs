use std::fs;

use ed25519_compact::{KeyPair, Seed};

use super::*;

const WORKFLOW: &str = include_str!("../../../../../.github/workflows/release.yml");
const TITLE_LINE: &str = "# trame v1.2.3";
const PROGRAM: &[u8] = b"MZ program";

fn version() -> Version {
    Version::parse("1.2.3").expect("a version")
}

fn key_pair(seed: u8) -> KeyPair {
    KeyPair::from_seed(Seed::new([seed; 32]))
}

fn list(title: &str, separator: &str) -> Vec<u8> {
    let hash = digest(PROGRAM).expect("a digest");
    format!("{title}\n{hash}{separator}trame.exe\n").into_bytes()
}

fn signed_by(keys: &KeyPair, list: Vec<u8>) -> io::Result<SignedList> {
    let signature = keys.sk.sign(&list, None).to_vec();
    SignedList::checked(list, signature, &keys.pk, version())
}

#[test]
fn accepts_an_asset_whose_hash_is_listed() {
    for separator in ["  ", " *"] {
        let signed = signed_by(&key_pair(1), list(TITLE_LINE, separator)).expect("a signed list");
        let expected = signed.listed("trame.exe").expect("the program is listed");
        assert!(confirm(&expected, PROGRAM).is_ok(), "{separator:?}");
    }
}

#[test]
fn rejects_an_asset_whose_bytes_changed() {
    let signed = signed_by(&key_pair(1), list(TITLE_LINE, "  ")).expect("a signed list");
    let expected = signed.listed("trame.exe").expect("the program is listed");
    let refused = confirm(&expected, &b"MZ another program"[..]).expect_err("a changed file");
    assert_eq!(refused.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn rejects_an_asset_missing_from_the_signed_list() {
    let signed = signed_by(&key_pair(1), list(TITLE_LINE, "  ")).expect("a signed list");
    assert!(signed.listed("trame-1.2.3-setup.exe").is_err());
    assert!(signed.listed("trame").is_err());
}

#[test]
fn rejects_a_list_with_a_bad_signature() {
    let keys = key_pair(1);
    let signature = keys.sk.sign(list(TITLE_LINE, "  "), None).to_vec();
    let mut changed = list(TITLE_LINE, "  ");
    changed[TITLE_LINE.len() + 1] ^= 1;
    let checked = |list, signature: &[u8], key| {
        SignedList::checked(list, signature.to_vec(), key, version()).is_err()
    };
    assert!(checked(changed, &signature, &keys.pk));
    assert!(checked(list(TITLE_LINE, "  "), &signature[1..], &keys.pk));
    assert!(checked(list(TITLE_LINE, "  "), &signature, &key_pair(2).pk));
}

#[test]
fn rejects_a_list_signed_for_another_version() {
    assert!(signed_by(&key_pair(1), list("# trame v1.2.2", "  ")).is_err());
    assert!(signed_by(&key_pair(1), list("", "  ")).is_err());
}

#[test]
fn a_saved_list_loads_back_for_its_version() {
    let folder = std::env::temp_dir().join(format!("trame-signed-{}", std::process::id()));
    fs::create_dir_all(&folder).expect("scratch folder");
    let keys = key_pair(1);
    let signed = signed_by(&keys, list(TITLE_LINE, "  ")).expect("a signed list");
    signed.save(&folder).expect("the list is saved");
    let loaded = SignedList::opened(&folder, &keys.pk, version());
    fs::remove_dir_all(&folder).expect("cleanup");
    assert!(loaded.and_then(|signed| signed.listed("trame.exe")).is_ok());
}

#[test]
fn the_release_key_is_an_ed25519_public_key() {
    assert!(release_key().is_ok());
}

#[test]
fn the_release_signs_the_list_the_app_reads() {
    assert!(WORKFLOW.contains(&format!("{TITLE}$GITHUB_REF_NAME")));
    assert!(WORKFLOW.contains(LIST) && WORKFLOW.contains(SIGNATURE));
    assert!(WORKFLOW.contains("secrets.TRAME_SIGNING_KEY"));
}
