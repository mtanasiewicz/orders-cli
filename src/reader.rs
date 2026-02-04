use crate::order::Order;
use crate::statistics::Statistics;
use csv::ReaderBuilder;
use std::fs::File;

pub fn read_csv(file_path: &str) -> Result<Statistics, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;

    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .buffer_capacity(1024 * 1024)
        .from_reader(file);

    let mut statistics = Statistics::new();

    for (i, result) in reader.records().enumerate() {
        match result {
            Ok(record) => match Order::from_csv_record(&record) {
                Ok(order) => statistics.accept(order),
                Err(err) => statistics.add_error(i + 1, err),
            },
            Err(_) => {
                statistics.add_error(i + 1, crate::order::ParseError::InvalidFormat);
            }
        }
    }

    Ok(statistics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::ParseError;

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

    #[test]
    fn read_csv_handles_invalid_lines() {
        use std::fs::write;

        let path = "test_invalid.csv";
        write(path, "1,John,10,paid\ninvalid_line\n3,Bob,-30,refunded\n4,Mike,20,unknown\n").unwrap();

        let stats = read_csv(path).unwrap();
        assert_eq!(stats.error_count(), 3);

        let errors = stats.errors();
        assert_eq!(errors[0], (2, &ParseError::InvalidFormat));
        assert_eq!(errors[1], (3, &ParseError::InvalidPrice));
        assert_eq!(errors[2], (4, &ParseError::InvalidStatus));

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_csv_handles_empty_file() {
        use std::fs::write;

        let path = "test_empty.csv";
        write(path, "").unwrap();

        let stats = read_csv(path).unwrap();
        assert_eq!(stats.error_count(), 0);

        std::fs::remove_file(path).unwrap();
    }
}
