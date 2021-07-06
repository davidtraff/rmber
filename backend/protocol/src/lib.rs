mod packet;
mod value;
mod error_code;

use std::convert::TryInto;
use rand::{Fill, Rng};
use std::hash::Hash;
use std::io::{Error, ErrorKind};

pub use packet::*;
pub use value::*;
pub use error_code::*;

pub trait Key: Sized {
    fn from_slice(key: &[u8]) -> Result<Self, Error>;
    fn as_slice(&self) -> &[u8];
}

#[derive(Debug, PartialEq, Eq)]
pub struct StringKey(String);

impl StringKey {
    pub fn new(value: &str) -> Result<Self, Error> {
        if !value.is_ascii() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid key. Only ascii-characters allowed",
            ));
        }

        Ok(StringKey(value.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn get_string(self) -> String {
        self.0
    }
}

impl Key for StringKey {
    fn from_slice(key: &[u8]) -> Result<Self, Error> {
        match std::str::from_utf8(key) {
            Ok(key) => StringKey::new(key),
            Err(e) => Err(Error::new(ErrorKind::InvalidInput, e)),
        }
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Hash for StringKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawKey<const LEN: usize>([u8; LEN]);

impl<const LEN: usize> RawKey<LEN> {
    pub fn new_random() -> Result<Self, Error> {
        let mut rng = rand::thread_rng();
        let mut key = RawKey([0u8; LEN]);

        match key.try_fill(&mut rng) {
            Ok(_) => Ok(key),
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
    }
}

impl<const LEN: usize> Fill for RawKey<LEN> {
    fn try_fill<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), rand::Error> {
        rng.try_fill_bytes(&mut self.0)
    }
}

impl<const LEN: usize> Key for RawKey<LEN> {
    fn from_slice(key: &[u8]) -> Result<Self, Error> {
        if key.len() != LEN {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid key length. Expected {} bytes but got {}",
                    LEN,
                    key.len()
                ),
            ));
        }

        let clone = key.clone();

        Ok(RawKey(clone.try_into().unwrap()))
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl<const LEN: usize> std::fmt::Display for RawKey<LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, len={})", hex::encode(self.0), LEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_key_from_slice() {
        assert_eq!(
            StringKey::from_slice(b"test").expect("Could not parse").0,
            "test"
        );
        assert_eq!(
            StringKey::from_slice(b"TEST").expect("Could not parse").0,
            "test"
        );

        assert_eq!(
            StringKey::from_slice(b"tEsTtEsTtEsTtEsTtEsTtEsTtEsT")
                .expect("Could not parse")
                .0,
            "testtesttesttesttesttesttest"
        );

        assert!(StringKey::from_slice("💖".as_bytes()).is_err());
    }

    #[test]
    fn string_key_as_slice() {
        assert_eq!(
            StringKey::new("test").unwrap().as_slice(),
            "test".as_bytes()
        );
        assert_eq!(
            StringKey::new("TEST").unwrap().as_slice(),
            "test".as_bytes()
        );
    }
}
