use std::str::FromStr;

use anyhow::bail;
use derive_more::{AsRef, Display, Into};
use regex::Regex;

#[derive(Display, AsRef, Debug, Clone, PartialEq, Eq, Default)]
pub struct Name(String);

impl FromStr for Name {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = Regex::new(r#"\A[0-9a-z\-]{3,43}\z"#)?;
        if r.is_match(s) {
            return Ok(Name(s.into()));
        }

        bail!("Invalid name format")
    }
}

#[cfg(test)]
mod tests {
    use std::{any, collections::HashMap};

    use super::*;

    #[test]
    fn test_valid() {
        let s: anyhow::Result<Name> = "smith".parse();
        assert_eq!(s.unwrap(), Name("smith".to_string()))
    }

    #[test]
    fn test_invalid() {
        let s: anyhow::Result<Name> = "Smith".parse();
        assert!(s.is_err())
    }
}
