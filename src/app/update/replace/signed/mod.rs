#[cfg(test)]
mod tests;

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use ed25519_compact::sha512::Hash;
use ed25519_compact::{PublicKey, Signature};

use super::super::download::{self, Progress};
use super::super::version::Version;

pub const LIST: &str = "SHA512SUMS";
pub const SIGNATURE: &str = "SHA512SUMS.sig";
const TITLE: &str = "# trame ";
const RELEASE_KEY: &[u8] = include_bytes!("../../../../../packaging/release-key.der");
const ED25519_KEY_PREFIX: [u8; 12] = [
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

pub struct SignedList {
    list: Vec<u8>,
    signature: Vec<u8>,
}

pub fn fetch(
    version: Version,
    asset: &str,
    target: &Path,
    shown: Progress,
) -> io::Result<SignedList> {
    let signed = SignedList::download(version)?;
    download::download(version, asset, target, shown)?;
    signed.check(asset, target)?;
    Ok(signed)
}

impl SignedList {
    fn download(version: Version) -> io::Result<Self> {
        let list = download::read(version, LIST)?;
        let signature = download::read(version, SIGNATURE)?;
        Self::checked(list, signature, &release_key()?, version)
    }

    pub fn load(folder: &Path, version: Version) -> io::Result<Self> {
        Self::opened(folder, &release_key()?, version)
    }

    fn opened(folder: &Path, key: &PublicKey, version: Version) -> io::Result<Self> {
        let list = fs::read(folder.join(LIST))?;
        let signature = fs::read(folder.join(SIGNATURE))?;
        Self::checked(list, signature, key, version)
    }

    fn checked(
        list: Vec<u8>,
        signature: Vec<u8>,
        key: &PublicKey,
        version: Version,
    ) -> io::Result<Self> {
        let signed = Signature::from_slice(&signature)
            .and_then(|signature| key.verify(&list, &signature))
            .is_ok();
        let title = format!("{TITLE}{version}");
        let first_line = list.split(|byte| *byte == b'\n').next();
        match (signed, first_line == Some(title.as_bytes())) {
            (true, true) => Ok(Self { list, signature }),
            (false, _) => Err(refused("the list of checksums has a bad signature")),
            (true, false) => Err(refused("the list of checksums is for another version")),
        }
    }

    pub fn save(&self, folder: &Path) -> io::Result<()> {
        fs::write(folder.join(LIST), &self.list)?;
        fs::write(folder.join(SIGNATURE), &self.signature)
    }

    pub fn listed(&self, asset: &str) -> io::Result<String> {
        String::from_utf8_lossy(&self.list)
            .lines()
            .find_map(|line| hash_of(line, asset))
            .ok_or_else(|| refused(format!("{asset} is not in the list of checksums")))
    }

    pub fn check(&self, asset: &str, file: &Path) -> io::Result<()> {
        confirm(&self.listed(asset)?, File::open(file)?)
    }
}

pub fn confirm(expected: &str, reader: impl Read) -> io::Result<()> {
    match digest(reader)? == expected {
        true => Ok(()),
        false => Err(refused("the download does not match its signed checksum")),
    }
}

pub fn digest(mut reader: impl Read) -> io::Result<String> {
    let mut hashing = Hashing(Hash::new());
    io::copy(&mut reader, &mut hashing)?;
    let hash = hashing.0.finalize();
    Ok(hash.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn hash_of(line: &str, asset: &str) -> Option<String> {
    let (hash, name) = line.split_once(' ')?;
    let name = name.strip_prefix([' ', '*'])?;
    (name == asset).then(|| hash.to_owned())
}

fn release_key() -> io::Result<PublicKey> {
    RELEASE_KEY
        .strip_prefix(&ED25519_KEY_PREFIX)
        .and_then(|raw| PublicKey::from_slice(raw).ok())
        .ok_or_else(|| refused("the release key is not an ed25519 key"))
}

fn refused(reason: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason.into())
}

struct Hashing(Hash);

impl Write for Hashing {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
