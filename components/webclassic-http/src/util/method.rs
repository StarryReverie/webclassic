use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use snafu::Snafu;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Head,
    Put,
    Delete,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Head => write!(f, "HEAD"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
        }
    }
}

#[derive(Debug, Snafu)]
pub enum ParseMethodError {
    #[snafu(display("unknown HTTP method: {method}"))]
    Unknown { method: String },
}

impl FromStr for Method {
    type Err = ParseMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "HEAD" => Ok(Method::Head),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            _ => Err(ParseMethodError::Unknown {
                method: s.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Method::Get.to_string(), "GET");
        assert_eq!(Method::Post.to_string(), "POST");
        assert_eq!(Method::Delete.to_string(), "DELETE");
    }

    #[test]
    fn from_str_uppercase() {
        assert_eq!("GET".parse::<Method>().unwrap(), Method::Get);
        assert_eq!("POST".parse::<Method>().unwrap(), Method::Post);
    }

    #[test]
    fn from_str_case_insensitive() {
        assert_eq!("get".parse::<Method>().unwrap(), Method::Get);
        assert_eq!("Post".parse::<Method>().unwrap(), Method::Post);
    }

    #[test]
    fn from_str_unknown() {
        assert!("PATCH".parse::<Method>().is_err());
        assert!("foobar".parse::<Method>().is_err());
    }
}
