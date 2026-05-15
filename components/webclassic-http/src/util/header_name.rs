use std::fmt::{Display, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use snafu::Snafu;

#[derive(Debug, Clone)]
pub struct HeaderName {
    inner: String,
}

impl HeaderName {
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl PartialEq for HeaderName {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq_ignore_ascii_case(&other.inner)
    }
}

impl Eq for HeaderName {}

impl Hash for HeaderName {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        for b in self.inner.bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
    }
}

impl Display for HeaderName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Snafu)]
pub enum ParseHeaderNameError {
    #[snafu(display("empty header name"))]
    Empty,
    #[snafu(display("invalid character in header name: {byte}"))]
    InvalidByte { byte: u8 },
}

impl FromStr for HeaderName {
    type Err = ParseHeaderNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseHeaderNameError::Empty);
        }
        for b in s.bytes() {
            if !is_header_name_token(b) {
                return Err(ParseHeaderNameError::InvalidByte { byte: b });
            }
        }
        Ok(HeaderName {
            inner: s.to_ascii_lowercase(),
        })
    }
}

fn is_header_name_token(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn parse_valid() {
        let name: HeaderName = "Content-Type".parse().unwrap();
        assert_eq!(name.as_str(), "content-type");
    }

    #[test]
    fn parse_rejects_empty() {
        assert!("".parse::<HeaderName>().is_err());
    }

    #[test]
    fn parse_rejects_invalid_byte() {
        assert!("Bad Header".parse::<HeaderName>().is_err());
        assert!("Header:Name".parse::<HeaderName>().is_err());
    }

    #[test]
    fn case_insensitive_equality() {
        let a: HeaderName = "content-type".parse().unwrap();
        let b: HeaderName = "Content-Type".parse().unwrap();
        let c: HeaderName = "CONTENT-TYPE".parse().unwrap();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn case_insensitive_hash() {
        let mut map = HashMap::new();
        let a: HeaderName = "content-type".parse().unwrap();
        let b: HeaderName = "Content-Type".parse().unwrap();
        map.insert(a, "value");
        assert_eq!(map.get(&b), Some(&"value"));
    }
}
