mod order;
mod statistics;

use crate::order::Order;
use crate::statistics::Statistics;
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
    let file = File::open(Args::parse().file)?;
    let reader = BufReader::new(file);

    let mut errors: HashMap<usize, String> = HashMap::new();
    let mut statistics: Statistics = Statistics::new();

    for (i, line) in reader.lines().enumerate() {
        let line_content = line?;

        match Order::from_str(&line_content) {
            Ok(order) => statistics.accept(order),
            Err(error) => {
                errors.insert(i + 1, format!("Error: {}", error));
            }
        };
    }

    println!("{}", statistics);
    println!("{:?}", errors);

    Ok(())
}
