use std::fmt::{Display, Formatter};
use std::collections::{HashSet};
use std::hash::{Hash};
use std::io::{Cursor, Read};
use std::cmp::{max, min};

use git2::{Buf, Commit, Repository, Signature};
use gnuplot::*;

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

fn is_capitalized(word: &str) -> bool {
    word.len() > 0 && word.chars().next().unwrap().is_uppercase()
}

#[derive(Debug)]
pub struct Stats {
    pub commit_summery: String,
    pub inserts: usize,
    pub deletes: usize,
    pub signed: bool,
    pub timestamp: i64
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
        let timestamp = commit.time().seconds();
        Self { commit_summery, inserts, deletes, signed, timestamp }
    }

    fn compute_magic(summery_len: usize) -> i32 {
        // put the length of the summery through a magic function such that ideally < 50 chars
        min((-((summery_len as f32 - 25.0) / 6.0) *
              ((summery_len as f32 - 25.0) / 6.0) + 15.0) as i32, 10)
    }

    pub fn score_loss(&self) -> i32 {
        let punctuation_chars: Vec<&str> = vec![".", "!", "?", ",", ";"];
        let common_words_table = HashSet::from([
            "added",   "adds",
            "removed", "removes",
            "fixed",   "fixes",
            "moved",   "moves",
            "merged",  "merges",
            "updated",  "updates",
        ]);

        let mut sum: i32 = 0;
        // grade the punctuation of the commit message
        for p in punctuation_chars {
            sum += self.commit_summery.matches(p).count() as i32;
        }

        // check if is not written in imperative form
        let split = self.commit_summery.split_whitespace();
        for w in split {
            if common_words_table.contains(w) {
                sum += 1;
            }
        }

        // bound the result of the previous evaluations
        sum = min(sum, 5) * 4;

        // check if the first character of the commit summery is capitalized
        if !is_capitalized(&self.commit_summery) {
            sum += 3;
        }
        if !self.signed { sum += 5; }
        let magic = Stats::compute_magic(self.commit_summery.len());
        if magic < 0 { sum += magic; }
        // strongly discourage large commits
        sum += ((self.inserts as f32).powf(0.55) - (self.inserts as f32).sqrt()) as i32;
        sum
    }

    pub fn score_gain(&self) -> i32 {
        let common_words_table = HashSet::from([
            "add", "remove", "fix", "move", "merge", "update"
        ]);

        let mut sum: i32 = 0;
        // check if is written in imperative form
        let split = self.commit_summery.split_whitespace();
        for w in split {
            if common_words_table.contains(w) {
                sum += 1;
            }
        }

        // bound the result of the previous evaluation
        sum = min(sum, 2) * 3;

        // check if the first character of the commit summery is capitalized
        if is_capitalized(&self.commit_summery) {
            sum += 3;
        }

        if self.signed { sum += 10; }
        let magic = Stats::compute_magic(self.commit_summery.len());
        if magic > 0 { sum += magic; }
        sum + (self.inserts as f32).sqrt() as i32
    }

    pub fn score(&self) -> i32 {
        max(self.score_gain() - self.score_loss(), 0)
    }
}

pub fn plot_gain_loss(nice: bool, x: Vec<i64>, y1: Vec<i32>, y2: Vec<i32>) {
    let y_0 = vec![0; x.len()];
    let mut fg = Figure::new();
    fg.set_pre_commands("set colorsequence classic")
      .set_title("Score Loss Comparison");
    if !nice { fg.set_terminal("dumb ansi256", ""); }
    fg.axes2d()
      .set_x_ticks(Some((AutoOption::from(Auto), 0)), &[Format("%d/%m")], &[])
      .set_x_time(true)
      .set_x_label("Date", &[])
      .set_y_label("Points", &[])
      .set_x_range(AutoOption::from(Fix(x[0] as f64)),
                   AutoOption::from(Fix(x[x.len() - 1] as f64)))
      .fill_between(&x, &y_0, &y1, &[FillRegion(Below), Color("black"), Caption("Score")])
      .fill_between(&x, &y1, &y2, &[FillRegion(Below), Color("red"), Caption("Score Loss")]);
    fg.show().unwrap();
}
