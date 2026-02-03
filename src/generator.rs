use crate::order::OrderStatus;
use rand::Rng;
use std::fs::File;
use std::io::{BufWriter, Write};

const FIRST_NAMES: &[&str] = &[
    "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Ivy", "Jack",
    "Kate", "Leo", "Mia", "Noah", "Olivia", "Paul", "Quinn", "Rose", "Sam", "Tina",
];

const LAST_NAMES: &[&str] = &[
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Wilson",
    "Moore", "Taylor", "Anderson", "Thomas", "Jackson", "White", "Harris", "Martin", "Clark",
];

pub fn generate_csv(output_path: &str, size_mb: f64) -> Result<(), Box<dyn std::error::Error>> {
    let target_bytes = (size_mb * 1024.0 * 1024.0) as u64;
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    let mut bytes_written: u64 = 0;
    let mut id: u32 = 1;
    let mut rng = rand::rng();

    while bytes_written < target_bytes {
        let line = generate_line(&mut rng, id);
        let line_bytes = line.as_bytes();
        writer.write_all(line_bytes)?;
        bytes_written += line_bytes.len() as u64;
        id += 1;
    }

    writer.flush()?;

    println!(
        "Generated {} orders ({:.2} MB) to {}",
        id - 1,
        bytes_written as f64 / 1024.0 / 1024.0,
        output_path
    );

    Ok(())
}

fn generate_line(rng: &mut impl Rng, id: u32) -> String {
    let customer = generate_customer_name(rng);
    let amount = generate_amount(rng);
    let status = generate_status(rng);

    format!("{},{},{:.2},{}\n", id, customer, amount, status)
}

fn generate_customer_name(rng: &mut impl Rng) -> String {
    let first = FIRST_NAMES[rng.random_range(0..FIRST_NAMES.len())];
    let last = LAST_NAMES[rng.random_range(0..LAST_NAMES.len())];
    format!("{} {}", first, last)
}

fn generate_amount(rng: &mut impl Rng) -> f64 {
    rng.random_range(0.01..10000.0)
}

fn generate_status(rng: &mut impl Rng) -> OrderStatus {
    match rng.random_range(0..100) {
        0..=69 => OrderStatus::Paid,
        70..=89 => OrderStatus::Cancelled,
        _ => OrderStatus::Refunded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufRead, BufReader};

    #[test]
    fn generates_file_with_approximate_size() {
        let output_path = "/tmp/test_generate.csv";
        let size_mb = 0.01;

        generate_csv(output_path, size_mb).unwrap();

        let metadata = fs::metadata(output_path).unwrap();
        let actual_size = metadata.len() as f64 / 1024.0 / 1024.0;

        assert!(actual_size >= size_mb);
        assert!(actual_size < size_mb + 0.001);

        fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn generates_valid_csv_lines() {
        let output_path = "/tmp/test_generate_valid.csv";

        generate_csv(output_path, 0.001).unwrap();

        let file = File::open(output_path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.split(',').collect();
            assert_eq!(parts.len(), 4);

            let id: u32 = parts[0].parse().unwrap();
            assert!(id > 0);

            let amount: f64 = parts[2].parse().unwrap();
            assert!(amount >= 0.01);
            assert!(amount <= 10000.0);

            let status = parts[3].to_lowercase();
            assert!(status == "paid" || status == "cancelled" || status == "refunded");
        }

        fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn generates_sequential_ids() {
        let output_path = "/tmp/test_generate_ids.csv";

        generate_csv(output_path, 0.001).unwrap();

        let file = File::open(output_path).unwrap();
        let reader = BufReader::new(file);

        let mut expected_id: u32 = 1;
        for line in reader.lines() {
            let line = line.unwrap();
            let id: u32 = line.split(',').next().unwrap().parse().unwrap();
            assert_eq!(id, expected_id);
            expected_id += 1;
        }

        fs::remove_file(output_path).unwrap();
    }
}
