use std::str::FromStr;

use anyhow::anyhow;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyVal(String, String);

impl KeyVal {
    pub fn pair(self) -> (String, String) {
        (self.0, self.1)
    }
}

impl FromStr for KeyVal {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, val) = s
            .split_once('=')
            .ok_or_else(|| anyhow!("Invalid key=value"))?;
        Ok(KeyVal(key.to_string(), val.to_string()))
    }
}
