use std::{any, str::from_utf8};

use bitcoin::hashes::hex::ToHex;

use crate::{
    db::{self, names_nsid, NamespaceModel},
    util::Nsid,
};

pub fn list_namespaces() -> anyhow::Result<()> {
    println!("Listing namespaces:");
    let nstree = db::namespaces()?;
    for result in nstree.into_iter() {
        let (nsid, nsdoc) = result?;
        let nsid = nsid.to_hex();
        let namespace = NamespaceModel::decode(&nsdoc)?;
        println!("{namespace:?}");
    }

    Ok(())
}
pub fn names_index() -> anyhow::Result<()> {
    println!("Listing names -> nsid:");
    let names_nsid = names_nsid()?;
    for result in names_nsid.iter() {
        let (name, nsid) = result?;
        let name = from_utf8(&name)?;
        let nsid = nsid.to_hex();
        println!("{name} => {nsid}");
    }

    Ok(())
}
