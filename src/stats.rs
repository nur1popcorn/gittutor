use std::hash::{Hash, Hasher};
use std::io::{Cursor, Read};

use git2::Buf;

use pgp::armor::Dearmor;
use pgp::types::KeyId;
use pgp::{Deserializable, StandaloneSignature};

#[derive(Hash, Eq, PartialEq, Debug)]
struct Author {
    name: String,
    email: String,
    key_id: Option<[u8; 8]>
}

fn get_issuer_key_id(buf: Buf) -> Option<[u8; 8]> {
    // extract the raw signature data
    let mut dearmor = Dearmor::new(Cursor::new(buf.as_ref()));
    let mut bytes = Vec::new();
    dearmor.read_to_end(&mut bytes).ok()?;

    // parse the signature and read the issuer
    let sig = StandaloneSignature::from_bytes(Cursor::new(bytes)).ok()?;
    <[u8; 8]>::try_from(sig.signature.issuer()?.as_ref().clone()).ok()
}
