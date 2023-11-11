use bitcoin::secp256k1::XOnlyPublicKey;

use crate::Hash160;

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
    use super::*;

    #[test]
    fn test_nsid_builder() {
        let pk: XOnlyPublicKey = "60de6fbc4a78209942c62706d904ff9592c2e856f219793f7f73e62fc33bfc18"
            .parse()
            .unwrap();
        let nsid = NsidBuilder::new("hello-world", &pk).finalize();

        assert_eq!(
            nsid,
            "273968a1e7be2ef0acbcae6f61d53e73101e2983".parse().unwrap()
        )
    }
}
