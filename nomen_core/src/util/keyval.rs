use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyVal(String, String);

impl KeyVal {
    pub fn pair(self) -> (String, String) {
        (self.0, self.1)
    }
}

impl FromStr for KeyVal {
    type Err = super::UtilError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, val) = s
            .split_once('=')
            .ok_or_else(|| super::UtilError::InvalidKeyVal(s.to_string()))?;
        Ok(KeyVal(key.to_string().to_uppercase(), val.to_string()))
    }
}
