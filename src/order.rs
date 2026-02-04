use std::fmt::Display;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u32,
    pub customer: String,
    pub amount: f64,
    pub status: OrderStatus,
}

impl Order {
    pub fn from_csv_record(record: &csv::StringRecord) -> Result<Order, ParseError> {
        if record.len() != 4 {
            return Err(ParseError::InvalidFormat);
        }

        let amount = record[2]
            .parse::<f64>()
            .map_err(|_| ParseError::InvalidPrice)?;

        if amount < 0.0 {
            return Err(ParseError::InvalidPrice);
        }

        Ok(Order {
            id: record[0]
                .parse::<u32>()
                .map_err(|_| ParseError::InvalidId)?,
            customer: record[1].to_string(),
            amount,
            status: OrderStatus::from_str(&record[3])?,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter, Display)]
pub enum OrderStatus {
    Paid,
    Cancelled,
    Refunded,
}

impl OrderStatus {
    pub fn from_str(status_str: &str) -> Result<Self, ParseError> {
        match status_str.to_lowercase().as_str() {
            "paid" => Ok(OrderStatus::Paid),
            "cancelled" => Ok(OrderStatus::Cancelled),
            "refunded" => Ok(OrderStatus::Refunded),
            _ => Err(ParseError::InvalidStatus),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ParseError {
    InvalidId,
    InvalidPrice,
    InvalidStatus,
    InvalidFormat,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidId => write!(f, "Invalid order id"),
            ParseError::InvalidPrice => write!(f, "Invalid price"),
            ParseError::InvalidStatus => write!(f, "Invalid status"),
            ParseError::InvalidFormat => write!(
                f,
                "Invalid format, line should contain exactly 4 comma separated values."
            ),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(fields: Vec<&str>) -> csv::StringRecord {
        let mut record = csv::StringRecord::new();
        for field in fields {
            record.push_field(field);
        }
        record
    }

    #[test]
    fn parses_valid_record() {
        let record = make_record(vec!["2", "customer", "10.0", "paid"]);
        let order = Order::from_csv_record(&record).unwrap();

        assert_eq!(order.id, 2);
        assert_eq!(order.customer, "customer");
        assert!((order.amount - 10.0).abs() < f64::EPSILON);
        assert_eq!(order.status, OrderStatus::Paid);
    }

    #[test]
    fn parses_low_price() {
        let record = make_record(vec!["1", "customer", "0.0", "paid"]);
        let order = Order::from_csv_record(&record).unwrap();
        assert!((order.amount - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fails_on_invalid_id() {
        let record = make_record(vec!["invalid", "customer", "10.0", "paid"]);
        let result = Order::from_csv_record(&record);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidId);
    }

    #[test]
    fn fails_on_invalid_price() {
        let record = make_record(vec!["1", "customer", "invalid", "paid"]);
        let result = Order::from_csv_record(&record);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidPrice);
    }

    #[test]
    fn fails_on_negative_price() {
        let record = make_record(vec!["1", "customer", "-10.0", "paid"]);
        let result = Order::from_csv_record(&record);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidPrice);
    }

    #[test]
    fn fails_on_invalid_status() {
        let record = make_record(vec!["1", "customer", "10.0", "invalid"]);
        let result = Order::from_csv_record(&record);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidStatus);
    }

    #[test]
    fn parses_status_case_insensitive() {
        let record = make_record(vec!["1", "customer", "10.0", "PaId"]);
        let order = Order::from_csv_record(&record).unwrap();
        assert_eq!(order.status, OrderStatus::Paid);
    }

    #[test]
    fn fails_on_wrong_field_count() {
        let record = make_record(vec!["1", "customer"]);
        let result = Order::from_csv_record(&record);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidFormat);
    }
}
