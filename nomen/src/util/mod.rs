mod keyval;

pub use keyval::*;

use time::{macros::format_description, OffsetDateTime};

pub fn format_time(timestamp: i64) -> anyhow::Result<String> {
    let dt = OffsetDateTime::from_unix_timestamp(timestamp)?;
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    Ok(dt.format(format)?)
}