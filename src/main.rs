mod stats;

use std::cmp::min;
use std::collections::HashMap;
use clap::Parser;
use git2::Repository;

use crate::stats::*;

/// A simple program to judge your usage of git
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sets a specific author by name or email
    #[arg(short, long)]
    author: Option<String>,


    /// The path to the repository which should be scored
    #[arg(group = "input", default_value_t = String::from("."))]
    input_path: String,

    /// Indicates if the used issuer key id is unique false by default
    #[arg(long, default_value_t = false)]
    issuer: bool
    // detail flag provides graphs for author
    // list good commits i.e. commits with a high score
}

fn main() {
    let args: Args = Args::parse();
    let repo = Repository::open(args.input_path)
        .expect("Failed to locate the git repository.");

    // construct a revwalk to iterate over commit graph
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();

    let mut result_vec: Vec<(Author, Stats)> = Vec::new();
    for oid in revwalk {
        let oid = oid.unwrap();
        let commit = repo.find_commit(oid).unwrap();
        let signature = repo.extract_signature(&oid, None);
        let stats = Stats::from(&repo, &commit, signature.is_ok());
        let author = Author::new(commit.author(),
            if args.issuer { signature.ok() } else { None });
        result_vec.push((author, stats));
    }

    let mut result_map = HashMap::new();
    for (author, stats) in result_vec {
        *result_map.entry(author).or_insert(0) += stats.score();
    }

    let mut result_vec = Vec::new();
    for p in result_map {
        result_vec.push(p)
    }
    result_vec.sort_by(|a, b| { b.1.cmp(&a.1) });
    for i in 0 .. min(result_vec.len(), 10) {
        let author = &result_vec[i].0;
        let name = &author.name;
        let email = &author.email;
        let score = &result_vec[i].1;
        println!("#{} {name} {email} {score}", i + 1);
    }
}
