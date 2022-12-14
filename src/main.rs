mod stats;

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
    input_path: String
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
        let author = Author::new(commit.author(), signature.ok());
        result_vec.push((author, stats));
    }

    result_vec.sort_by(|a, b| { a.1.score().cmp(&b.1.score()) });
    println!("{:?}", result_vec);
}
