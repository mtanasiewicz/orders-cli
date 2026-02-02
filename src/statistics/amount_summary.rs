use crate::order::Order;
use crate::statistics::Stat;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use std::fmt::{Display, Formatter};

pub struct AmountSummary {
    amounts: Vec<f64>,
    min: Option<f64>,
    max: Option<f64>,
    sum: f64,
}

impl AmountSummary {
    pub fn new() -> Self {
        Self {
            amounts: Vec::new(),
            min: None,
            max: None,
            sum: 0.0,
        }
    }

    fn count(&self) -> usize {
        self.amounts.len()
    }

    fn min(&self) -> f64 {
        self.min.unwrap_or(0.0)
    }

    fn max(&self) -> f64 {
        self.max.unwrap_or(0.0)
    }

    fn mean(&self) -> f64 {
        if self.amounts.is_empty() {
            0.0
        } else {
            self.sum / self.amounts.len() as f64
        }
    }

    fn median(&self) -> f64 {
        if self.amounts.is_empty() {
            return 0.0;
        }

        let mut sorted = self.amounts.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted.len();
        if len % 2 == 0 {
            (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
        } else {
            sorted[len / 2]
        }
    }

    fn std_dev(&self) -> f64 {
        if self.amounts.len() < 2 {
            return 0.0;
        }

        let mean = self.mean();
        let variance: f64 = self
            .amounts
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (self.amounts.len() - 1) as f64;

        variance.sqrt()
    }
}

impl Stat for AmountSummary {
    fn accept(&mut self, order: &Order) {
        self.amounts.push(order.amount);
        self.sum += order.amount;

        self.min = Some(match self.min {
            Some(current) => current.min(order.amount),
            None => order.amount,
        });

        self.max = Some(match self.max {
            Some(current) => current.max(order.amount),
            None => order.amount,
        });
    }
}

impl Display for AmountSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- Amount Summary ---")?;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec!["Metric", "Value"]);
        table.add_row(vec!["Count".to_string(), self.count().to_string()]);
        table.add_row(vec!["Min".to_string(), format!("{:.2}", self.min())]);
        table.add_row(vec!["Max".to_string(), format!("{:.2}", self.max())]);
        table.add_row(vec!["Mean".to_string(), format!("{:.2}", self.mean())]);
        table.add_row(vec!["Median".to_string(), format!("{:.2}", self.median())]);
        table.add_row(vec!["Std Dev".to_string(), format!("{:.2}", self.std_dev())]);

        writeln!(f, "{table}")?;
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
    fn new_summary_is_empty() {
        let summary = AmountSummary::new();

        assert_eq!(summary.count(), 0);
        assert!((summary.min() - 0.0).abs() < f64::EPSILON);
        assert!((summary.max() - 0.0).abs() < f64::EPSILON);
        assert!((summary.mean() - 0.0).abs() < f64::EPSILON);
        assert!((summary.median() - 0.0).abs() < f64::EPSILON);
        assert!((summary.std_dev() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn accept_tracks_count() {
        let mut summary = AmountSummary::new();

        summary.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        summary.accept(&create_order(2, "Jane", 200.0, OrderStatus::Paid));

        assert_eq!(summary.count(), 2);
    }

    #[test]
    fn accept_tracks_min() {
        let mut summary = AmountSummary::new();

        summary.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        summary.accept(&create_order(2, "Jane", 50.0, OrderStatus::Paid));
        summary.accept(&create_order(3, "Bob", 200.0, OrderStatus::Paid));

        assert!((summary.min() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn accept_tracks_max() {
        let mut summary = AmountSummary::new();

        summary.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        summary.accept(&create_order(2, "Jane", 50.0, OrderStatus::Paid));
        summary.accept(&create_order(3, "Bob", 200.0, OrderStatus::Paid));

        assert!((summary.max() - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mean_calculates_correctly() {
        let mut summary = AmountSummary::new();

        summary.accept(&create_order(1, "John", 10.0, OrderStatus::Paid));
        summary.accept(&create_order(2, "Jane", 20.0, OrderStatus::Paid));
        summary.accept(&create_order(3, "Bob", 30.0, OrderStatus::Paid));

        assert!((summary.mean() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn median_odd_count() {
        let mut summary = AmountSummary::new();

        summary.accept(&create_order(1, "John", 30.0, OrderStatus::Paid));
        summary.accept(&create_order(2, "Jane", 10.0, OrderStatus::Paid));
        summary.accept(&create_order(3, "Bob", 20.0, OrderStatus::Paid));

        assert!((summary.median() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn median_even_count() {
        let mut summary = AmountSummary::new();

        summary.accept(&create_order(1, "John", 10.0, OrderStatus::Paid));
        summary.accept(&create_order(2, "Jane", 20.0, OrderStatus::Paid));
        summary.accept(&create_order(3, "Bob", 30.0, OrderStatus::Paid));
        summary.accept(&create_order(4, "Alice", 40.0, OrderStatus::Paid));

        assert!((summary.median() - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn std_dev_calculates_correctly() {
        let mut summary = AmountSummary::new();

        // Values: 10, 20, 30 -> mean = 20
        // Variance = ((10-20)^2 + (20-20)^2 + (30-20)^2) / 2 = (100 + 0 + 100) / 2 = 100
        // Std Dev = sqrt(100) = 10
        summary.accept(&create_order(1, "John", 10.0, OrderStatus::Paid));
        summary.accept(&create_order(2, "Jane", 20.0, OrderStatus::Paid));
        summary.accept(&create_order(3, "Bob", 30.0, OrderStatus::Paid));

        assert!((summary.std_dev() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn std_dev_single_value_is_zero() {
        let mut summary = AmountSummary::new();

        summary.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));

        assert!((summary.std_dev() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn render_contains_header() {
        let summary = AmountSummary::new();
        let output = summary.to_string();

        assert!(output.contains("--- Amount Summary ---"));
    }
}
