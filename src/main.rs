mod diff_builder;
mod file;

use crate::diff_builder::DiffBuilder;
use crate::file::FileReader;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    source: String,
    target: String,
}

fn main() {
    let args = Args::parse();

    let source_file = FileReader::read(args.source).unwrap();
    let target_file = FileReader::read(args.target).unwrap();

    let diff = DiffBuilder::build(source_file, target_file);

    println!("{}", diff);
}
