use std::borrow::Cow;

use bitcoin::XOnlyPublicKey;

use crate::util::Hash160;

use super::Nsid;

pub struct NsidBuilder {
    root_name: String,
    pk: XOnlyPublicKey,
}

impl NsidBuilder {
    pub fn new(root_name: &str, root_pk: &XOnlyPublicKey) -> NsidBuilder {
        NsidBuilder {
            root_name: root_name.to_owned(),
            pk: *root_pk,
        }
    }

    pub fn finalize(self) -> Nsid {
        let mut hasher = Hash160::default();
        hasher.update(self.root_name.as_bytes());
        hasher.update(&self.pk.serialize());
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_nsid_builder() {
        let pk: XOnlyPublicKey = "d57b873363d2233d3cd54453416deff9546df50d963bb1208da37f10a4c23d6f"
            .parse()
            .unwrap();
        let nsid = NsidBuilder::new("smith", &pk).finalize();

        assert_eq!(
            nsid,
            "073e4dd94dd1128b964d45c2fba2fe4aca124df5".parse().unwrap()
        )
    }
}
