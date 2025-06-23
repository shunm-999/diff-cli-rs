use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    source: String,
    target: String,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world!");
}
