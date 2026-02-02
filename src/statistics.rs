mod amount_by_status;
mod amount_distribution;
mod amount_summary;
mod conversion_metrics;
mod customer_risk_profile;
mod top_orders;

use crate::order::{Order, ParseError};
use crate::statistics::amount_by_status::AmountsByStatus;
use crate::statistics::amount_distribution::AmountDistribution;
use crate::statistics::amount_summary::AmountSummary;
use crate::statistics::conversion_metrics::ConversionMetrics;
use crate::statistics::customer_risk_profile::CustomerRiskProfile;
use crate::statistics::top_orders::TopOrders;
use comfy_table::Table;
use std::fmt::{Display, Formatter};

trait Stat: Display {
    fn accept(&mut self, order: &Order);
}

pub struct Statistics {
    stats: Vec<Box<dyn Stat>>,
    errors: Vec<LineError>,
}

struct LineError {
    line_number: usize,
    error: ParseError,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
            stats: vec![
                Box::new(AmountsByStatus::new()),
                Box::new(AmountDistribution::new()),
                Box::new(AmountSummary::new()),
                Box::new(ConversionMetrics::new()),
                Box::new(TopOrders::new()),
                Box::new(CustomerRiskProfile::new()),
            ],
            errors: Vec::new(),
        }
    }

    pub fn accept(&mut self, order: Order) {
        for stat in &mut self.stats {
            stat.accept(&order)
        }
    }

    pub fn add_error(&mut self, line_number: usize, error: ParseError) {
        self.errors.push(LineError { line_number, error });
    }

    #[cfg(test)]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    #[cfg(test)]
    pub fn errors(&self) -> Vec<(usize, &ParseError)> {
        self.errors
            .iter()
            .map(|e| (e.line_number, &e.error))
            .collect()
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Orders statistics ===")?;
        writeln!(f)?;

        for stat in &self.stats {
            write!(f, "{}", stat)?;
        }

        let mut errors_table = Table::new();
        errors_table.set_header(vec!["Line number", "Error"]);
        for error in &self.errors {
            errors_table.add_row(vec![error.line_number.to_string(), error.error.to_string()]);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::OrderStatus;

    fn create_order(id: u32, customer: &str, amount: f64, status: OrderStatus) -> Order {
        Order {
            id,
            customer: customer.to_string(),
            amount,
            status,
        }
    }

    #[test]
    fn new_statistics_has_no_errors() {
        let stats = Statistics::new();

        assert_eq!(stats.error_count(), 0);
    }

    #[test]
    fn add_error_records_errors() {
        let mut stats = Statistics::new();

        stats.add_error(1, ParseError::InvalidId);
        stats.add_error(5, ParseError::InvalidPrice);

        assert_eq!(stats.error_count(), 2);
        let errors = stats.errors();
        assert_eq!(errors[0], (1, &ParseError::InvalidId));
        assert_eq!(errors[1], (5, &ParseError::InvalidPrice));
    }

    #[test]
    fn accept_does_not_increase_error_count() {
        let mut stats = Statistics::new();

        stats.accept(create_order(1, "John", 10.0, OrderStatus::Paid));
        stats.accept(create_order(2, "Jane", 20.0, OrderStatus::Cancelled));

        assert_eq!(stats.error_count(), 0);
    }

    #[test]
    fn display_includes_header() {
        let stats = Statistics::new();

        let output = format!("{}", stats);

        assert!(output.contains("=== Orders statistics ==="));
    }

    #[test]
    fn display_includes_amounts_by_status_section() {
        let mut stats = Statistics::new();
        stats.accept(create_order(1, "John", 10.0, OrderStatus::Paid));

        let output = format!("{}", stats);

        assert!(output.contains("--- Amounts by status ---"));
        assert!(output.contains("Paid"));
        assert!(output.contains("10.00"));
    }
}
