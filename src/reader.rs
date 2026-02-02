use crate::order::Order;
use crate::statistics::Statistics;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_csv(file_path: &str) -> Result<Statistics, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    read_from(reader)
}

pub fn read_from<R: BufRead>(reader: R) -> Result<Statistics, Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::ParseError;
    use std::io::Cursor;

    #[test]
    fn read_from_parses_valid_orders_without_errors() {
        let cursor = Cursor::new(
            "1,John,10.00,Paid
                   2,Jane,20.00,Cancelled
                   3,Bob,30.00,Refunded
                   4,Michael,25.00,Paid",
        );

        let stats = read_from(cursor).unwrap();

        assert_eq!(stats.error_count(), 0);
    }

    #[test]
    fn read_from_records_errors_for_invalid_lines() {
        let cursor = Cursor::new(
            "1,John,10.00,Paid
                   invalid_line
                   3,Bob,-30.00,Refunded
                   4,Michael,25.00,unknown_status",
        );

        let stats = read_from(cursor).unwrap();

        assert_eq!(stats.error_count(), 3);
        let errors = stats.errors();
        assert_eq!(errors[0], (2, &ParseError::InvalidFormat));
        assert_eq!(errors[1], (3, &ParseError::InvalidPrice));
        assert_eq!(errors[2], (4, &ParseError::InvalidStatus));
    }

    #[test]
    fn read_from_records_correct_line_numbers_for_errors() {
        let cursor = Cursor::new(
            "1,John,10.00,Paid
                   2,Jane,20.00,Cancelled
                   invalid,Bob,30.00,Refunded",
        );

        let stats = read_from(cursor).unwrap();

        assert_eq!(stats.error_count(), 1);
        let errors = stats.errors();
        assert_eq!(errors[0].0, 3);
        assert_eq!(errors[0].1, &ParseError::InvalidId);
    }

    #[test]
    fn read_from_handles_empty_input() {
        let cursor = Cursor::new("");

        let stats = read_from(cursor).unwrap();

        assert_eq!(stats.error_count(), 0);
    }

    #[test]
    fn read_csv_reads_real_file() {
        use std::fs::write;

        let path = "test_read_csv.csv";

        write(path, "1,John,10,paid\n2,Jane,20,cancelled\n").unwrap();

        let stats = read_csv(path).unwrap();

        assert_eq!(stats.error_count(), 0);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_csv_returns_error_for_missing_file() {
        let result = read_csv("nonexistent_file.csv");

        assert!(result.is_err());
    }
}
