use bitcoin::hashes::hex::ToHex;

use crate::db::{self, Namespace};

pub fn list_namespaces() -> anyhow::Result<()> {
    println!("Listing namespaces:");
    let nstree = db::namespaces()?;
    for result in nstree.into_iter() {
        let (nsid, nsdoc) = result?;
        let nsid = nsid.to_hex();
        let namespace = Namespace::decode(&nsdoc)?;
        println!("{namespace:?}");
    }

    Ok(())
}
