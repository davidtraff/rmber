mod value;
mod packet;

pub use value::*;
pub use packet::*;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub inner: Option<Box<dyn std::error::Error>>,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error {
            message: String::from(message),
            inner: None,
        }
    }

    pub fn new_with_inner(message: &str, inner: Box<dyn std::error::Error>) -> Self {
        Error {
            message: String::from(message),
            inner: Some(inner),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::new_with_inner("IO-error", Box::new(e))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::new_with_inner("UTF8-error", Box::new(e))
    }
}

pub trait Key: Sized {
    fn from_slice(key: &[u8]) -> Result<Self, Error>;
    fn as_slice(&self) -> &[u8];
}

#[derive(Debug, PartialEq, Eq)]
pub struct StringKey(String);

impl StringKey {
    pub fn new(value: &str) -> Result<Self, Error> {
        if !value.is_ascii() {
            return Err(Error::new("Invalid key. Only ascii-characters allowed"));
        }

        Ok(StringKey(value.to_lowercase()))
    }
}

impl Key for StringKey {
    fn from_slice(key: &[u8]) -> Result<Self, Error> {
        match std::str::from_utf8(key) {
            Ok(key) => StringKey::new(key),
            Err(e) => Err(Error::new_with_inner(
                "Invalid key. Only ascii-characters allowed",
                Box::new(e),
            )),
        }
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_bytes()
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
