use clap::Parser;

/// A simple program to judge your usage of git
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sets a specific author
    #[arg(short, long)]
    author: Option<String>,

    /// Sets a cutoff date
    #[arg(short, long)]
    date: Option<u8>,
}

fn main() {
    let args = Args::parse();
}
