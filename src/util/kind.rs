use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, bail};
use secp256k1::XOnlyPublicKey;

use super::{Hash160, Nsid, NsidBuilder};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NomenKind {
    Create,
}

impl From<NomenKind> for u8 {
    fn from(value: NomenKind) -> Self {
        match value {
            NomenKind::Create => 0x00,
        }
    }
}

impl Display for NomenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NomenKind::Create => "create",
        };
        write!(f, "{s}")
    }
}

impl FromStr for NomenKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(NomenKind::Create),
            _ => Err(anyhow!("Unrecognized Nomen transaction type")),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CreateV0 {
    pub fingerprint: [u8; 5],
    pub nsid: Nsid,
}

impl CreateV0 {
    fn create(fingerprint: [u8; 5], nsid: Nsid) -> CreateV0 {
        CreateV0 { fingerprint, nsid }
    }

    fn parse_create(value: &[u8]) -> anyhow::Result<CreateV0> {
        Ok(CreateV0::create(
            value[..5].try_into()?,
            value[5..].try_into()?,
        ))
    }
}

impl TryFrom<&[u8]> for CreateV0 {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // TODO: refactor be closer to CreateV1 try_from
        if !value.starts_with(b"NOM") {
            bail!("Not an Nomen transaction")
        }
        let value = &value[3..];

        if !value.starts_with(&[0x00]) {
            bail!("Unsupported Nomen version")
        }
        let value = &value[1..];

        let kind = match value.first() {
            Some(0x00) => CreateV0::parse_create(&value[1..])?,
            _ => bail!("Unexpected blockchain tx type"),
        };

        Ok(kind)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateV1 {
    pub pubkey: XOnlyPublicKey,
    pub name: String,
}
impl CreateV1 {
    pub fn create(pubkey: XOnlyPublicKey, name: &str) -> CreateV1 {
        CreateV1 {
            pubkey,
            name: name.to_owned(),
        }
    }

    pub fn parse_create(value: &[u8]) -> anyhow::Result<CreateV1> {
        // TODO: verify name validity
        Ok(CreateV1 {
            pubkey: XOnlyPublicKey::from_slice(&value[..32])?,
            name: String::from_utf8(value[32..].to_vec())?,
        })
    }

    pub fn fingerprint(&self) -> [u8; 5] {
        Hash160::default()
            .chain_update(self.name.as_bytes())
            .fingerprint()
    }

    pub fn nsid(&self) -> Nsid {
        NsidBuilder::new(&self.name, &self.pubkey).finalize()
    }
}

impl TryFrom<&[u8]> for CreateV1 {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if !value.starts_with(b"NOM\x01") {
            bail!("Not an Nomen V1 Create transaction")
        }
        let value = &value[4..];

        let kind = match value.first() {
            Some(0x00) => CreateV1::parse_create(&value[1..])?,
            _ => bail!("Unexpected blockchain tx type"),
        };

        Ok(kind)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_parse_create_v0() {
        let fp = hex::decode("0102030405").unwrap();
        let nsid = Nsid::from_str("c215a040e1c3566deb8ef3d37e2a4915cd9ba672").unwrap();
        let create = b"NOM\x00\x00"
            .iter()
            .chain(fp.iter())
            .chain(nsid.to_vec().iter())
            .copied()
            .collect_vec();
        assert_eq!(
            CreateV0::try_from(create.as_ref()).unwrap(),
            CreateV0::create(fp.try_into().unwrap(), nsid)
        );
    }

    #[test]
    fn test_parse_create_v1() {
        let pk = hex::decode("285d4ca25cbe209832aa15a4b94353b877a2fe6c3b94dee1a4c8bc36770304db")
            .unwrap();
        let pk = XOnlyPublicKey::from_slice(&pk).unwrap();
        let create = b"NOM\x01\x00"
            .iter()
            .chain(pk.serialize().iter())
            .chain(b"hello-world".iter())
            .copied()
            .collect_vec();
        assert_eq!(
            CreateV1::try_from(create.as_slice()).unwrap(),
            CreateV1::create(pk, "hello-world")
        );
    }

    #[test]
    fn test_invalid_version() {
        let wrong_ver = b"NOM\x01\x00";
        assert!(CreateV0::try_from(wrong_ver.as_ref()).is_err())
    }

    #[test]
    fn test_invalid_tx_type() {
        let wrong_ver = b"NOZ\x00\x00";
        assert!(CreateV0::try_from(wrong_ver.as_ref()).is_err())
    }

    #[test]
    fn test_invalid_tx_kind() {
        let wrong_ver = b"NOM\x00\x10";
        assert!(CreateV0::try_from(wrong_ver.as_ref()).is_err())
    }
}
