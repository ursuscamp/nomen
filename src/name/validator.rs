use anyhow::anyhow;
use regex::Regex;

use super::Name;

pub struct Validator<'a> {
    name: &'a Name,
}

impl<'a> Validator<'a> {
    pub fn new(name: &'a Name) -> Self {
        Validator { name }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        self.validate_characters()?;

        Ok(())
    }

    fn validate_characters(&self) -> anyhow::Result<()> {
        let re = Regex::new(r"\A[a-z][a-z0-9\-_]*\z").expect("regex should compile");

        let n = &self.name.name;
        if !re.is_match(n) {
            return Err(anyhow!("Invalid character in name '{n}'"));
        }

        for name in &self.name.names {
            Validator::new(name).validate_characters()?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_character() -> anyhow::Result<()> {
        let mut name = Name::default();
        name.name = "com".into();
        Validator::new(&name).validate()?;

        name.name = "com!".into();
        assert!(Validator::new(&name).validate().is_err());

        Ok(())
    }
}
