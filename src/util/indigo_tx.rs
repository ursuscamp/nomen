use anyhow::bail;

use super::Nsid;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IndigoKind {
    Create,
    Update,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IndigoTx {
    pub kind: IndigoKind,
    pub nsid: Nsid,
}

impl IndigoTx {
    fn create(nsid: Nsid) -> IndigoTx {
        IndigoTx {
            kind: IndigoKind::Create,
            nsid,
        }
    }

    fn update(nsid: Nsid) -> IndigoTx {
        IndigoTx {
            kind: IndigoKind::Update,
            nsid,
        }
    }

    fn parse_create(value: &[u8]) -> anyhow::Result<IndigoTx> {
        Ok(IndigoTx::create(value.try_into()?))
    }

    fn parse_update(value: &[u8]) -> anyhow::Result<IndigoTx> {
        Ok(IndigoTx::update(value.try_into()?))
    }
}

impl From<IndigoKind> for u8 {
    fn from(value: IndigoKind) -> Self {
        match value {
            IndigoKind::Create => 0x00,
            IndigoKind::Update => 0x01,
        }
    }
}

impl TryFrom<&[u8]> for IndigoTx {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if !value.starts_with(b"IND") {
            bail!("Not an Indigo transaction")
        }
        let value = &value[3..];

        if !value.starts_with(&[0x00]) {
            bail!("Unsupported Indigo version")
        }
        let value = &value[1..];

        let kind = match value.first() {
            Some(0x00) => IndigoTx::parse_create(&value[1..])?,
            Some(0x01) => IndigoTx::parse_update(&value[1..])?,
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
        let create = b"IND\x00\x00"
            .iter()
            .chain(nsid.to_vec().iter())
            .copied()
            .collect_vec();
        assert_eq!(
            IndigoTx::try_from(create.as_ref()).unwrap(),
            IndigoTx::create(nsid)
        );
    }

    #[test]
    fn test_parse_update() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let update = b"IND\x00\x01"
            .iter()
            .chain(nsid.to_vec().iter())
            .copied()
            .collect_vec();
        assert_eq!(
            IndigoTx::try_from(update.as_ref()).unwrap(),
            IndigoTx::update(nsid)
        );
    }

    #[test]
    fn test_invalid_version() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let wrong_ver = b"IND\x01\x00";
        assert!(IndigoTx::try_from(wrong_ver.as_ref()).is_err())
    }

    #[test]
    fn test_invalid_tx_type() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let wrong_ver = b"INZ\x00\x00";
        assert!(IndigoTx::try_from(wrong_ver.as_ref()).is_err())
    }

    #[test]
    fn test_invalid_tx_kind() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let wrong_ver = b"IND\x00\x10";
        assert!(IndigoTx::try_from(wrong_ver.as_ref()).is_err())
    }
}
