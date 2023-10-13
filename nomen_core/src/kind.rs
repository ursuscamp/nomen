use std::{fmt::Display, str::FromStr};

use nostr_sdk::{EventBuilder, UnsignedEvent};
use secp256k1::{schnorr::Signature, XOnlyPublicKey};

use crate::Name;

use super::{CreateBuilder, Hash160, Nsid, NsidBuilder, TransferBuilder};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NomenKind {
    Create,
    Transfer,
}

impl From<NomenKind> for u8 {
    fn from(value: NomenKind) -> Self {
        match value {
            NomenKind::Create => 0x00,
            NomenKind::Transfer => 0x01,
        }
    }
}

impl Display for NomenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NomenKind::Create => "create",
            NomenKind::Transfer => "transfer",
        };
        write!(f, "{s}")
    }
}

impl FromStr for NomenKind {
    type Err = super::UtilError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(NomenKind::Create),
            "transfer" => Ok(NomenKind::Transfer),
            _ => Err(super::UtilError::NomenKind(s.to_string())),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CreateV0 {
    pub fingerprint: [u8; 5],
    pub nsid: Nsid,
}

impl CreateV0 {
    pub fn create(fingerprint: [u8; 5], nsid: Nsid) -> CreateV0 {
        CreateV0 { fingerprint, nsid }
    }

    pub fn parse_create(value: &[u8]) -> Result<CreateV0, super::UtilError> {
        Ok(CreateV0::create(
            value[..5].try_into()?,
            value[5..].try_into()?,
        ))
    }

    pub fn serialize(&self) -> Vec<u8> {
        b"NOM\x00\x00"
            .iter()
            .chain(self.fingerprint.iter())
            .chain(self.nsid.iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for CreateV0 {
    type Error = super::UtilError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if !value.starts_with(b"NOM\x00") {
            return Err(super::UtilError::UnexpectedNomenTxType);
        }
        let value = &value[4..];

        match value.first() {
            Some(0x00) => Ok(CreateV0::parse_create(&value[1..])?),
            _ => Err(super::UtilError::UnexpectedNomenTxType),
        }
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

    pub fn fingerprint(&self) -> [u8; 5] {
        Hash160::default()
            .chain_update(self.name.as_bytes())
            .fingerprint()
    }

    pub fn nsid(&self) -> Nsid {
        NsidBuilder::new(&self.name, &self.pubkey).finalize()
    }

    pub fn parse_create(value: &[u8]) -> Result<CreateV1, super::UtilError> {
        let name = String::from_utf8(value[32..].to_vec())?;
        let _ = Name::from_str(&name)?;
        Ok(CreateV1 {
            pubkey: XOnlyPublicKey::from_slice(&value[..32])?,
            name,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        b"NOM\x01\x00"
            .iter()
            .chain(self.pubkey.serialize().iter())
            .chain(self.name.as_bytes().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for CreateV1 {
    type Error = super::UtilError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if !value.starts_with(b"NOM\x01") {
            return Err(super::UtilError::UnexpectedNomenTxType);
        }
        let value = &value[4..];

        match value.first() {
            Some(0x00) => Ok(CreateV1::parse_create(&value[1..])?),
            _ => Err(super::UtilError::UnexpectedNomenTxType),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferV1 {
    pub pubkey: XOnlyPublicKey,
    pub name: String,
}
impl TransferV1 {
    pub fn create(pubkey: XOnlyPublicKey, name: &str) -> TransferV1 {
        TransferV1 {
            pubkey,
            name: name.to_owned(),
        }
    }

    pub fn parse_create(value: &[u8]) -> Result<TransferV1, super::UtilError> {
        let name = String::from_utf8(value[32..].to_vec())?;
        let _ = Name::from_str(&name)?;
        Ok(TransferV1 {
            pubkey: XOnlyPublicKey::from_slice(&value[..32])?,
            name,
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

    pub fn serialize(&self) -> Vec<u8> {
        b"NOM\x01\x01"
            .iter()
            .chain(self.pubkey.serialize().iter())
            .chain(self.name.as_bytes().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for TransferV1 {
    type Error = super::UtilError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if !value.starts_with(b"NOM\x01") {
            return Err(super::UtilError::UnexpectedNomenTxType);
        }
        let value = &value[4..];

        match value.first() {
            Some(0x01) => Ok(TransferV1::parse_create(&value[1..])?),
            _ => Err(super::UtilError::UnexpectedNomenTxType),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureV1 {
    pub signature: Signature,
}
impl SignatureV1 {
    pub fn create(signature: &Signature) -> SignatureV1 {
        SignatureV1 {
            signature: *signature,
        }
    }

    pub fn parse_create(value: &[u8]) -> Result<SignatureV1, super::UtilError> {
        Ok(SignatureV1 {
            signature: Signature::from_slice(value)?,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        b"NOM\x01\x02"
            .iter()
            .chain(self.signature.as_ref().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for SignatureV1 {
    type Error = super::UtilError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if !value.starts_with(b"NOM\x01") {
            return Err(super::UtilError::UnexpectedNomenTxType);
        }
        let value = &value[4..];

        match value.first() {
            Some(0x02) => Ok(SignatureV1::parse_create(&value[1..])?),
            _ => Err(super::UtilError::UnexpectedNomenTxType),
        }
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
