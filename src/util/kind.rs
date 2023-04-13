use std::fmt::Display;

use anyhow::bail;

use super::Nsid;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NomenKind {
    Create,
}

impl Display for NomenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NomenKind::Create => "create",
        };
        write!(f, "{s}")
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NomenTx {
    pub kind: NomenKind,
    pub nsid: Nsid,
}

impl NomenTx {
    fn create(nsid: Nsid) -> NomenTx {
        NomenTx {
            kind: NomenKind::Create,
            nsid,
        }
    }

    fn parse_create(value: &[u8]) -> anyhow::Result<NomenTx> {
        Ok(NomenTx::create(value.try_into()?))
    }
}

impl From<NomenKind> for u8 {
    fn from(value: NomenKind) -> Self {
        match value {
            NomenKind::Create => 0x00,
        }
    }
}

impl TryFrom<&[u8]> for NomenTx {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if !value.starts_with(b"NOM") {
            bail!("Not an Nomen transaction")
        }
        let value = &value[3..];

        if !value.starts_with(&[0x00]) {
            bail!("Unsupported Nomen version")
        }
        let value = &value[1..];

        let kind = match value.first() {
            Some(0x00) => NomenTx::parse_create(&value[1..])?,
            _ => bail!("Unexpected blockchain tx type"),
        };

        Ok(kind)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use itertools::Itertools;
    use ripemd::digest::crypto_common::Block;

    use super::*;

    #[test]
    fn test_parse_create() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let create = b"NOM\x00\x00"
            .iter()
            .chain(nsid.to_vec().iter())
            .copied()
            .collect_vec();
        assert_eq!(
            NomenTx::try_from(create.as_ref()).unwrap(),
            NomenTx::create(nsid)
        );
    }

    #[test]
    fn test_invalid_version() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let wrong_ver = b"NOM\x01\x00";
        assert!(NomenTx::try_from(wrong_ver.as_ref()).is_err())
    }

    #[test]
    fn test_invalid_tx_type() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let wrong_ver = b"NOZ\x00\x00";
        assert!(NomenTx::try_from(wrong_ver.as_ref()).is_err())
    }

    #[test]
    fn test_invalid_tx_kind() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let wrong_ver = b"NOM\x00\x10";
        assert!(NomenTx::try_from(wrong_ver.as_ref()).is_err())
    }
}
