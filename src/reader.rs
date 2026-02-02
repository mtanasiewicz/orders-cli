use crate::order::Order;
use crate::statistics::Statistics;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_csv(file_path: &str) -> Result<Statistics, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut statistics = Statistics::new();

    for (i, line) in reader.lines().enumerate() {
        let line_content = line?;
        match Order::from_str(&line_content) {
            Ok(order) => statistics.accept(order),
            Err(err) => statistics.add_error(i + 1, err),
        }
    }

    Ok(statistics)
}
