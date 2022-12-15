extern crate core;

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
    /// The path to the repository which should be scored
    #[arg(group = "input", default_value_t = String::from("."))]
    input_path: String,
    /// Sets a specific author by name or email or issuer key id
    #[arg(short, long)]
    author: Option<String>,
    /// The number of authors which should be shown
    #[arg(short, default_value_t = 10)]
    n: usize,
    /// Set if the plot should not be written to stdout
    #[arg(long)]
    nice: bool,
    /// Compare the issuer key id of the commit signature
    #[arg(long)]
    issuer: bool,
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

    let mut stats_map: HashMap<&Author, i32> = HashMap::new();
    for (author, stats) in stats_vec.iter() {
        *stats_map.entry(author).or_insert(0) += stats.score();
    }

    let mut result_vec: Vec<(&&Author, &i32)> = stats_map.iter().collect();
    result_vec.sort_by(|a, b| { b.1.cmp(&a.1) });
    if args.author.is_some() {
        let pattern: String = args.author.unwrap().to_lowercase();
        for i in 0 .. result_vec.len() {
            let (author, score) = result_vec[i];
            if author.matches(&pattern) {
                println!("#{}\t({score})\t{author}", i + 1);

                // collect the data for plotting
                let mut s1 = 0; let mut s2 = 0;
                let mut x = Vec::new();
                let mut y1 = Vec::new(); let mut y2 = Vec::new();
                for (author_curr, stats) in stats_vec.iter() {
                    if *author != author_curr { continue }
                    let score = stats.score();
                    let loss = stats.score_loss();
                    s1 += score;
                    s2 += score + loss;
                    x.push(stats.timestamp);
                    y1.push(s1); y2.push(s2);
                }
                if x.len() > 1 { plot_gain_loss(args.nice, x, y1, y2); }
            }
        }
    } else {
        // print the top n commit authors
        for i in 0 .. min(result_vec.len(), args.n) {
            let (author, score) = result_vec[i];
            println!("#{}\t({score})\t{author}", i + 1);
        }
    }
}
