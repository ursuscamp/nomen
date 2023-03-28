use anyhow::bail;

use super::Nsid;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockchainKind {
    Create(Nsid),
    Update(Nsid, Nsid),
}
impl BlockchainKind {
    fn parse_create(value: &[u8]) -> anyhow::Result<BlockchainKind> {
        Ok(BlockchainKind::Create(value.try_into()?))
    }

    fn parse_update(value: &[u8]) -> anyhow::Result<BlockchainKind> {
        if value.len() < 40 {
            bail!("Invalid blockchain update tx: too few bytes")
        }
        let (from, to) = value.split_at(20);
        Ok(BlockchainKind::Update(from.try_into()?, to.try_into()?))
    }
}

impl TryFrom<&[u8]> for BlockchainKind {
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
            Some(0x00) => BlockchainKind::parse_create(&value[1..])?,
            Some(0x01) => BlockchainKind::parse_update(&value[1..])?,
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
            BlockchainKind::try_from(create.as_ref()).unwrap(),
            BlockchainKind::Create(nsid)
        );
    }

    #[test]
    fn test_parse_update() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let update = b"IND\x00\x01"
            .iter()
            .chain(nsid.to_vec().iter())
            .chain(nsid.to_vec().iter())
            .copied()
            .collect_vec();
        assert_eq!(
            BlockchainKind::try_from(update.as_ref()).unwrap(),
            BlockchainKind::Update(nsid, nsid)
        );
    }

    #[test]
    fn test_invalid_version() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let wrong_ver = b"IND\x01\x00";
        assert!(BlockchainKind::try_from(wrong_ver.as_ref()).is_err())
    }

    #[test]
    fn test_invalid_tx_type() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let wrong_ver = b"INZ\x00\x00";
        assert!(BlockchainKind::try_from(wrong_ver.as_ref()).is_err())
    }

    #[test]
    fn test_invalid_tx_kind() {
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let wrong_ver = b"IND\x00\x10";
        assert!(BlockchainKind::try_from(wrong_ver.as_ref()).is_err())
    }
}
