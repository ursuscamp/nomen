mod extractor;
mod hash160;
mod keyval;
mod kind;
mod name;
mod nostr;
mod nsid;
mod nsid_builder;
mod transfer;

use anyhow::bail;
pub use extractor::*;
pub use hash160::*;
pub use keyval::*;
pub use kind::*;
pub use name::*;
pub use nostr::*;
pub use nsid::*;
pub use nsid_builder::*;
use time::{macros::format_description, OffsetDateTime};
pub use transfer::*;

pub enum NameKind {
    Name = 38300,
}

impl From<NameKind> for nostr_sdk::Kind {
    fn from(value: NameKind) -> Self {
        nostr_sdk::Kind::ParameterizedReplaceable(value as u16)
    }
}

impl TryFrom<nostr_sdk::Kind> for NameKind {
    type Error = anyhow::Error;

    fn try_from(value: nostr_sdk::Kind) -> Result<Self, Self::Error> {
        let nk = match value {
            nostr_sdk::Kind::ParameterizedReplaceable(38300) => NameKind::Name,
            _ => bail!("Invalid Event kind"),
        };
        Ok(nk)
    }
}

pub fn format_time(timestamp: i64) -> anyhow::Result<String> {
    let dt = OffsetDateTime::from_unix_timestamp(timestamp)?;
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    Ok(dt.format(format)?)
}
