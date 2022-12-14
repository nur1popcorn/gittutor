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

    /// Indicates if the used issuer key id is unique
    #[arg(long, default_value_t = false)]
    issuer: bool,

    /// The number of authors which should be shown
    #[arg(short, default_value_t = 10)]
    n: usize

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

    let mut stats_vec: Vec<(Author, Stats)> = Vec::new();
    for oid in revwalk {
        let oid = oid.unwrap();
        let commit = repo.find_commit(oid).unwrap();
        let signature = repo.extract_signature(&oid, None);
        let stats = Stats::from(&repo, &commit, signature.is_ok());
        let author = Author::new(commit.author(),
            if args.issuer { signature.ok() } else { None });
        stats_vec.push((author, stats));
    }

    let mut stats_map = HashMap::new();
    for (author, stats) in stats_vec {
        *stats_map.entry(author).or_insert(0) += stats.score();
    }

    if args.author.is_some() {

    } else {
        // print the top n commit authors
        let mut result_vec: Vec<(&Author, &i32)> = stats_map.iter().collect();
        result_vec.sort_by(|a, b| { b.1.cmp(&a.1) });
        for i in 0 .. min(result_vec.len(), args.n) {
            let author = &result_vec[i].0;
            let name = &author.name;
            let email = &author.email;
            let score = &result_vec[i].1;
            println!("#{}\t({score})\t {name} {email}", i + 1);
        }
    }
}
