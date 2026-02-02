mod order;
mod reader;
mod statistics;

use crate::reader::read_csv;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version = "1.0",
    author = "Marcin Tanasiewicz",
    about = "Analyze orders in csv file"
)]
struct Args {
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = Args::parse().file;

    let statistics = read_csv(&file_name)?;
    println!("{}", statistics);

    Ok(())
}
