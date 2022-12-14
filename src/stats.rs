use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use std::hash::{Hash};
use std::io::{Cursor, Read};
use std::cmp::{max, min};

use git2::{Buf, Commit, Repository, Signature};

use pgp::armor::Dearmor;
use pgp::{Deserializable, StandaloneSignature};

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub key_id: Option<[u8; 8]>
}

impl Author {
    fn get_issuer_key_id(buf: &Buf) -> [u8; 8] {
        // extract the raw signature data
        let mut dearmor = Dearmor::new(Cursor::new(buf.as_ref()));
        let mut bytes = Vec::new();
        dearmor.read_to_end(&mut bytes).ok();

        // parse the signature and read the issuer
        let sig = StandaloneSignature::from_bytes(Cursor::new(bytes));
        <[u8; 8]>::try_from(sig.unwrap().signature.issuer().unwrap().as_ref()).unwrap()
    }

    pub fn new(signature: Signature, key_id: Option<(Buf, Buf)>) -> Self {
        Self {
            name: String::from(signature.name().unwrap()),
            email: String::from(signature.email().unwrap()),
            key_id: key_id.map(|key_id| { Author::get_issuer_key_id(&key_id.0) })
        }
    }

    pub fn matches(&self, pattern: &str) -> bool {
        let key_id = self.key_id.map(|key_id| {
            hex::encode(key_id).contains(pattern) });
        self.name.to_lowercase().contains(pattern) ||
        self.email.to_lowercase().contains(pattern) ||
        (key_id.is_some() && key_id.unwrap())
    }
}

impl Display for Author {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = &self.name;
        let email = &self.email;
        write!(f, "{name} {email}")
    }
}

#[derive(Debug)]
pub struct Stats {
    pub commit_summery: String,
    pub inserts: usize,
    pub deletes: usize,
    pub signed: bool
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

    fn score_commit_message(&self) -> i32 {
        let punctuation_chars: Vec<&str> = vec![".", "!", "?", ",", ";"];
        let common_words_table: HashMap<&str, i32> = HashMap::from([
            ("add",    1), ("added",   -1), ("adds",    -1),
            ("remove", 1), ("removed", -1), ("removes", -1),
            ("fix",    1), ("fixed",   -1), ("fixes",   -1),
            ("move",   1), ("moved",   -1), ("moves",   -1),
            ("merge",  1), ("merged",  -1), ("merges",  -1),
        ]);

        // grade the punctuation of the commit message
        let mut sum: i32 = 0;
        for p in punctuation_chars {
            sum -= self.commit_summery.matches(p).count() as i32;
        }
        // check if is written in imperative form
        for w in self.commit_summery.split(" ") {
            sum += common_words_table.get(w).unwrap_or(&0)
        }

        // bound the result of the previous evaluations
        sum = max(min(sum, 2), -2) * 3;

        // put the length of the summery through a magic function such that ideally < 50 chars
        let magic =
            - ((self.commit_summery.len() as f32 - 25.0) / 6.0) *
              ((self.commit_summery.len() as f32 - 25.0) / 6.0) + 15.0;
        sum + min(magic as i32, 10)
    }

    pub fn score(&self) -> i32 {
        max((self.inserts as f32).powf(0.69) as i32
            + self.score_commit_message()
            + if self.signed { 10 } else { 0 }, 0)
    }
}
