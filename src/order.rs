use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Order {
    id: u32,
    customer: String,
    amount: f64,
    status: OrderStatus,
}

impl Order {
    pub fn from_str(line: &str) -> Result<Order, ParseError> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(ParseError::InvalidFormat);
        }

        let amount = parts[2]
            .trim()
            .parse::<f64>()
            .map_err(|_| ParseError::InvalidPrice)?;

        if amount < 0.0 {
            return Err(ParseError::InvalidPrice);
        }

        let order = Order {
            id: parts[0]
                .trim()
                .parse::<u32>()
                .map_err(|_| ParseError::InvalidId)?,
            customer: parts[1].trim().to_string(),
            amount,
            status: OrderStatus::from_str(parts[3].trim())?,
        };

        Ok(order)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OrderStatus {
    Paid,
    Cancelled,
    Refunded,
}

impl OrderStatus {
    fn from_str(status_str: &str) -> Result<Self, ParseError> {
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
            ParseError::InvalidId => {
                write!(f, "Invalid order id")
            }
            ParseError::InvalidPrice => {
                write!(f, "Invalid price")
            }
            ParseError::InvalidStatus => {
                write!(f, "Invalid status")
            }
            ParseError::InvalidFormat => {
                write!(
                    f,
                    "Invalid format, line should contain exactly 4 comma separated values."
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_string() {
        let order = parse_order("2,customer,10.0,paid").unwrap();

        assert_eq!(order.id, 2);
        assert_eq!(order.customer, "customer");
        assert!((order.amount - 10.0).abs() < f64::EPSILON);
        assert_eq!(order.status, OrderStatus::Paid);
    }

    #[test]
    fn parses_trimmed_values() {
        let order = parse_order(" 2 , customer , 10.0 , paid ").unwrap();

        assert_eq!(order.id, 2);
        assert_eq!(order.customer, "customer");
        assert!((order.amount - 10.0).abs() < f64::EPSILON);
        assert_eq!(order.status, OrderStatus::Paid);
    }

    #[test]
    fn parses_low_price() {
        let order = parse_order("1,customer,0.0,paid").unwrap();

        assert_eq!(order.id, 1);
        assert_eq!(order.customer, "customer");
        assert!((order.amount - 0.0).abs() < f64::EPSILON);
        assert_eq!(order.status, OrderStatus::Paid);
    }

    #[test]
    fn fails_on_invalid_id() {
        let result = parse_order("invalid,customer,10.0,paid");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidId);
    }

    #[test]
    fn fails_on_invalid_price() {
        let result = parse_order("1,customer,invalid,paid");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidPrice);
    }

    #[test]
    fn fails_on_negative_price() {
        let result = parse_order("1,customer,-10.0,paid");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidPrice);
    }

    #[test]
    fn fails_on_invalid_status() {
        let result = parse_order("1,customer,10.0,invalid");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidStatus);
    }

    #[test]
    fn parses_status_case_insensitive() {
        let order = parse_order("1,customer,10.0,PaId").unwrap();

        assert_eq!(order.status, OrderStatus::Paid);
    }

    #[test]
    fn fails_on_invalid_string() {
        let result = parse_order("invalid string");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidFormat);
    }

    fn parse_order(s: &str) -> Result<Order, ParseError> {
        Order::from_str(s)
    }
}
