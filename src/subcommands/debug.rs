use std::{any, str::from_utf8};

use bitcoin::hashes::hex::ToHex;

use crate::{
    db::{self},
    util::Nsid,
};

pub fn list_namespaces() -> anyhow::Result<()> {
    println!("Listing namespaces:");

    Ok(())
}
pub fn names_index() -> anyhow::Result<()> {
    println!("Listing names -> nsid:");

    Ok(())
}
