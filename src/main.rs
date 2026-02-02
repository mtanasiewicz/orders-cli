mod generator;
mod order;
mod reader;
mod statistics;

use crate::generator::generate_csv;
use crate::reader::read_csv;
use clap::{Parser, Subcommand};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(
    version = "1.0",
    author = "Marcin Tanasiewicz",
    about = "Analyze orders in csv file"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Analyze orders from a CSV file
    Analyze {
        /// Path to the CSV file to analyze
        file: String,
    },
    /// Generate a CSV file with random order data
    Generate {
        /// Path for the output CSV file
        output: String,
        /// Size of the file to generate in megabytes
        #[arg(short, long)]
        size: f64,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let args = Args::parse();

    match args.command {
        Command::Analyze { file } => {
            let statistics = read_csv(&file)?;
            println!("{}", statistics);
        }
        Command::Generate { output, size } => {
            generate_csv(&output, size)?;
        }
    }

    println!("Total execution time: {:.2}s", start.elapsed().as_secs_f64());
    Ok(())
}
