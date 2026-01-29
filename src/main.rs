mod order;

use crate::order::Order;
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

    let mut orders: Vec<Order> = Vec::new();
    let mut errors: HashMap<usize, String> = HashMap::new();
    for (i, line) in reader.lines().enumerate() {
        let line_content = line?;

        match Order::from_str(&line_content) {
            Ok(order) => orders.push(order),
            Err(error) => {
                errors.insert(i + 1, format!("Error: {}", error));
            }
        };
    }

    println!("{:?}", orders);
    println!("{:?}", errors);

    Ok(())
}
