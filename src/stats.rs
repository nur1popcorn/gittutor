use std::hash::{Hash, Hasher};
use std::io::{Cursor, Read};

use git2::{Buf, Commit, Repository};

use pgp::armor::Dearmor;
use pgp::types::KeyId;
use pgp::{Deserializable, StandaloneSignature};

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Author {
    name: String,
    email: String,
    key_id: Option<[u8; 8]>
}

pub struct Stats {
    commit_summery: String,
    inserts: usize,
    deletes: usize,
    signed: bool
}

impl Stats {
    pub fn from(repo: &Repository, commit: &Commit, signed: bool) -> Self {
        let commit_summery = String::from(commit.summary().unwrap());
        let mut inserts = 0;
        let mut deletes = 0;
        if commit.parents().len() == 1 {
            // it is not a merge so collect the insert, delete data
            let tree = commit.tree().unwrap();

            let parent_id = commit.parent_id(0).unwrap();
            let parent_commit = repo.find_commit(parent_id).unwrap();
            let parent_tree = parent_commit.tree().unwrap();

            let diff = repo.diff_tree_to_tree(
                Some(&parent_tree), Some(&tree), None).unwrap();
            let stats = diff.stats().unwrap();

            inserts = stats.insertions();
            deletes = stats.deletions();
        }

        Self { commit_summery, inserts, deletes, signed }
    }
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
