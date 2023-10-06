use common::test_vectors;
use nomen_core::{CreateBuilder, Hash160, Name, NsidBuilder, SignatureV1, TransferV1};

mod common;

#[test]
fn test_nsid_builders() {
    let tv = test_vectors();
    for nv in tv.nsids {
        let nsid = NsidBuilder::new(&nv.name, &nv.pk).finalize();
        assert_eq!(nsid, nv.nsid);
    }
}

#[test]
fn test_hash_160() {
    let tv = test_vectors();
    for hash in tv.hash160s {
        let value = hash.value.as_bytes();
        let hash = hex::decode(&hash.hash).unwrap();
        let calc = Hash160::default().chain_update(value).finalize();
        assert_eq!(calc, hash.as_ref());
    }
}

#[test]
fn test_names() {
    let tv = test_vectors();
    for nv in tv.names {
        let n: Result<Name, _> = nv.name.parse();
        if nv.pass {
            assert!(n.is_ok());
        } else {
            assert!(n.is_err());
        }
    }
}

#[test]
fn test_op_returns() {
    let tv = test_vectors();
    for op_return in tv.op_returns {
        match op_return.kind.as_str() {
            "v0_create" => {
                let v0create =
                    CreateBuilder::new(&op_return.pk.unwrap(), &op_return.name).v0_op_return();
                assert_eq!(v0create, hex::decode(op_return.value).unwrap());
            }
            "v1_create" => {
                let v1create =
                    CreateBuilder::new(&op_return.pk.unwrap(), &op_return.name).v1_op_return();
                assert_eq!(v1create, hex::decode(op_return.value).unwrap());
            }
            "v1_transfer" => {
                let v1transfer = TransferV1 {
                    pubkey: op_return.pk.unwrap(),
                    name: op_return.name,
                };
                assert_eq!(
                    v1transfer.serialize(),
                    hex::decode(op_return.value).unwrap()
                );
            }
            "v1_signature" => {
                let v1sig = SignatureV1 {
                    signature: op_return.sig.unwrap(),
                }
                .serialize();
                assert_eq!(v1sig, hex::decode(op_return.value).unwrap());
            }
            _ => panic!(),
        }
    }
}
