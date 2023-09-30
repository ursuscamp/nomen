mod extractor;
mod hash160;
mod kind;
mod name;
mod nsid;
mod nsid_builder;
mod transfer;

pub use extractor::*;
pub use hash160::*;
pub use kind::*;
pub use name::*;
pub use nsid::*;
pub use nsid_builder::*;
use time::{macros::format_description, OffsetDateTime};
pub use transfer::*;

#[derive(thiserror::Error, Debug)]
pub enum UtilError {
    #[error("not a nomen transaction")]
    NotNomenError,
    #[error("unsupported nomen version")]
    UnsupportedNomenVersion,
    #[error("unexpectex tx type")]
    UnexpectedNomenTxType,
    #[error("name validation")]
    NameValidation,
    #[error("unknown nomen kind: {:?}", .0)]
    NomenKind(String),
    #[error("invalid Key=Value")]
    InvalidKeyVal(String),
    #[error("invalid event kind")]
    InvalidEventKind(nostr_sdk::Kind),
    #[error("nostr event signing error")]
    UnsignedEventError(#[from] nostr_sdk::event::unsigned::Error),
    #[error("slice conversion")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error("hex conversion")]
    HexDecode(#[from] hex::FromHexError),
    #[error("nostr key")]
    NostrKeyError(#[from] nostr_sdk::key::Error),
    #[error("regex")]
    RegexError(#[from] regex::Error),
    #[error("secp256k1")]
    Secp256k1Error(#[from] secp256k1::Error),
    #[error("string error")]
    StringError(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    ExtractorError(#[from] ExtractorError),
}

pub enum NameKind {
    Name = 38300,
}

impl From<NameKind> for nostr_sdk::Kind {
    fn from(value: NameKind) -> Self {
        nostr_sdk::Kind::ParameterizedReplaceable(value as u16)
    }
}

impl TryFrom<nostr_sdk::Kind> for NameKind {
    type Error = UtilError;

    fn try_from(value: nostr_sdk::Kind) -> Result<Self, Self::Error> {
        let nk = match value {
            nostr_sdk::Kind::ParameterizedReplaceable(38300) => NameKind::Name,
            _ => return Err(UtilError::InvalidEventKind(value)),
        };
        Ok(nk)
    }
}
