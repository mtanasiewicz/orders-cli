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
    use crate::order::OrderStatus;
    use std::io::Cursor;

    #[test]
    fn test_read_from() {
        let cursor = Cursor::new(
            "1,John,10.00,Paid
            2,Jane,20.00,Cancelled
            3,John,30.00,Refunded
            4,Michael, 25.00, Paid",
        );

        let stats = read_from(cursor).unwrap();

        assert_eq!(stats.count(OrderStatus::Paid), 2);
        assert_eq!(stats.average(OrderStatus::Paid), 17.5);
    }

    #[test]
    fn test_read_csv_reads_real_file() {
        use std::fs::write;

        let path = "test.csv";

        write(path, "1,John,10,paid\n").unwrap();

        let stats = read_csv(path).unwrap();

        assert_eq!(stats.count(OrderStatus::Paid), 1);
        assert_eq!(stats.total(OrderStatus::Paid), 10.0);

        std::fs::remove_file(path).unwrap();
    }
}
