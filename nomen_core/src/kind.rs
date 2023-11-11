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
    pub fn new(signature: &Signature) -> SignatureV1 {
        SignatureV1 {
            signature: *signature,
        }
    }

    pub fn parse_signature(value: &[u8]) -> Result<SignatureV1, super::UtilError> {
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
            Some(0x02) => Ok(SignatureV1::parse_signature(&value[1..])?),
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
    fn test_parse_create_serialize_v0() {
        let or =
            hex::decode("4e4f4d0000e5401df4b4273968a1e7be2ef0acbcae6f61d53e73101e2983").unwrap();
        let c = CreateV0::try_from(or.as_ref());
        assert!(c.is_ok());
        assert_eq!(c.unwrap().serialize(), or);
    }

    #[test]
    fn test_invalid_create_v0() {
        let or =
            hex::decode("4e4f4d0001e5401df4b4273968a1e7be2ef0acbcae6f61d53e73101e2983").unwrap();
        let c = CreateV0::try_from(or.as_ref());
        assert!(c.is_err());
    }

    #[test]
    fn test_parse_create_serialize_v1() {
        let pk = XOnlyPublicKey::from_str(
            "60de6fbc4a78209942c62706d904ff9592c2e856f219793f7f73e62fc33bfc18",
        )
        .unwrap();
        let or = hex::decode("4e4f4d010060de6fbc4a78209942c62706d904ff9592c2e856f219793f7f73e62fc33bfc1868656c6c6f2d776f726c64").unwrap();
        let c = CreateV1::try_from(or.as_ref());
        assert!(c.is_ok());
        assert_eq!(c.unwrap().serialize(), or);
    }

    #[test]
    fn test_invalid_create_v1() {
        let or = hex::decode(
            "4e4f4d010060de6fbc4a78209942c62706d904ff9592c2e856f219793f7f73e62fc33bfc186c64",
        )
        .unwrap();
        let c = CreateV1::try_from(or.as_ref());
        assert!(c.is_err());
    }

    #[test]
    fn test_parse_transfer_serialize_v1() {
        let or = hex::decode("4e4f4d010174301b9c5d30b764bca8d3eb4febb06862f558d292fde93b4a290d90850bac9168656c6c6f2d776f726c64").unwrap();
        let t = TransferV1::try_from(or.as_ref());
        assert!(t.is_ok());
        assert_eq!(t.unwrap().serialize(), or);
    }

    #[test]
    fn test_parse_signatuyre_serialize_v1() {
        let or = hex::decode("4e4f4d0102489e4e3ab29408da53733473156040a25e5a84cbca788c2b7143f971ead84192ae8bd8e4890cfabb08dca693875c28a1949ae0d13f5c6b08617e4fdc022bc751").unwrap();
        let t = SignatureV1::try_from(or.as_ref());
        assert!(t.is_ok());
        assert_eq!(t.unwrap().serialize(), or);
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
