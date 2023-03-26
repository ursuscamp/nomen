use std::str::FromStr;

use anyhow::anyhow;
use bitcoin::XOnlyPublicKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildPair(String, XOnlyPublicKey);

impl ChildPair {
    pub fn pair(self) -> (String, XOnlyPublicKey) {
        (self.0, self.1)
    }
}

impl FromStr for ChildPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, key) = s
            .split_once(':')
            .ok_or_else(|| anyhow!("Invalid child string"))?;
        Ok(ChildPair(name.to_string(), key.parse()?))
    }
}
