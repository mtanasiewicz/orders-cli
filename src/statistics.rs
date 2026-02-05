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

pub struct Statistics {
    amounts_by_status: AmountsByStatus,
    amount_distribution: AmountDistribution,
    amount_summary: AmountSummary,
    conversion_metrics: ConversionMetrics,
    top_orders: TopOrders,
    customer_risk_profile: CustomerRiskProfile,
    errors: Vec<LineError>,
}

struct LineError {
    line_number: usize,
    error: ParseError,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
            amounts_by_status: AmountsByStatus::new(),
            amount_distribution: AmountDistribution::new(),
            amount_summary: AmountSummary::new(),
            conversion_metrics: ConversionMetrics::new(),
            top_orders: TopOrders::new(),
            customer_risk_profile: CustomerRiskProfile::new(),
            errors: Vec::new(),
        }
    }

    pub fn accept(&mut self, order: Order) {
        self.amounts_by_status.accept(&order);
        self.amount_distribution.accept(&order);
        self.amount_summary.accept(&order);
        self.conversion_metrics.accept(&order);
        self.top_orders.accept(&order);
        self.customer_risk_profile.accept(&order);
    }

    pub fn merge(&mut self, other: Statistics) {
        self.amounts_by_status.merge(other.amounts_by_status);
        self.amount_distribution.merge(other.amount_distribution);
        self.amount_summary.merge(other.amount_summary);
        self.conversion_metrics.merge(other.conversion_metrics);
        self.top_orders.merge(other.top_orders);
        self.customer_risk_profile.merge(other.customer_risk_profile);
        self.errors.extend(other.errors);
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

        write!(f, "{}", self.amounts_by_status)?;
        write!(f, "{}", self.amount_distribution)?;
        write!(f, "{}", self.amount_summary)?;
        write!(f, "{}", self.conversion_metrics)?;
        write!(f, "{}", self.top_orders)?;
        write!(f, "{}", self.customer_risk_profile)?;

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

    #[test]
    fn merge_combines_statistics() {
        let mut stats1 = Statistics::new();
        stats1.accept(create_order(1, "John", 10.0, OrderStatus::Paid));

        let mut stats2 = Statistics::new();
        stats2.accept(create_order(2, "Jane", 20.0, OrderStatus::Paid));

        stats1.merge(stats2);

        let output = format!("{}", stats1);
        assert!(output.contains("30.00")); // total amount
    }

    #[test]
    fn merge_combines_errors() {
        let mut stats1 = Statistics::new();
        stats1.add_error(1, ParseError::InvalidId);

        let mut stats2 = Statistics::new();
        stats2.add_error(5, ParseError::InvalidPrice);

        stats1.merge(stats2);

        assert_eq!(stats1.error_count(), 2);
    }
}
