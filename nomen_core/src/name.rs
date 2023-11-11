use std::str::FromStr;

use derive_more::{AsRef, Display, Into};
use regex::Regex;

#[derive(Display, AsRef, Debug, Clone, PartialEq, Eq, Default)]
pub struct Name(String);

impl FromStr for Name {
    type Err = super::UtilError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = Regex::new(r#"\A[0-9a-z\-]{3,43}\z"#)?;
        if r.is_match(s) {
            return Ok(Name(s.into()));
        }

        Err(super::UtilError::NameValidation)
    }
}

#[cfg(test)]
mod tests {
    use std::{any, collections::HashMap};

    use crate::UtilError;

    use super::*;

    #[test]
    fn test_valid() {
        let r = ["hello-world", "123abc"]
            .into_iter()
            .map(Name::from_str)
            .all(|r| r.is_ok());
        assert!(r);
    }

    #[test]
    fn test_invalid() {
        let r = [
            "hello!",
            "ld",
            "abcdefghijklmnopqrztuvwxyzabcdefghijklmnopqrztuvwxyz",
        ]
        .into_iter()
        .map(Name::from_str)
        .all(|r| r.is_err());
        assert!(r);
    }
}
