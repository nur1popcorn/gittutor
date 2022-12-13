mod stats;

use std::env;
use std::path::PathBuf;

use clap::Parser;
use git2::Repository;

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

    for oid in revwalk {
        let oid = oid.unwrap();
        let commit = repo.find_commit(oid).unwrap();

    }
}
