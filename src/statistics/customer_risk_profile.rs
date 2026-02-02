use crate::order::{Order, OrderStatus};
use crate::statistics::Stat;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

const TOP_N: usize = 5;

#[derive(Default)]
struct CustomerData {
    paid_count: usize,
    cancelled_count: usize,
    refunded_count: usize,
    total_amount: f64,
}

impl CustomerData {
    fn total_orders(&self) -> usize {
        self.paid_count + self.cancelled_count + self.refunded_count
    }

    fn problem_rate(&self) -> f64 {
        let total = self.total_orders();
        if total > 0 {
            ((self.cancelled_count + self.refunded_count) as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    fn cancellation_rate(&self) -> f64 {
        let total = self.total_orders();
        if total > 0 {
            (self.cancelled_count as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    fn refund_rate(&self) -> f64 {
        let total = self.total_orders();
        if total > 0 {
            (self.refunded_count as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

pub struct CustomerRiskProfile {
    customers: HashMap<String, CustomerData>,
    min_orders_threshold: usize,
}

impl CustomerRiskProfile {
    pub fn new() -> Self {
        Self {
            customers: HashMap::new(),
            min_orders_threshold: 2,
        }
    }

    fn top_risky_customers(&self) -> Vec<(&String, &CustomerData)> {
        let mut customers: Vec<_> = self
            .customers
            .iter()
            .filter(|(_, data)| data.total_orders() >= self.min_orders_threshold)
            .collect();

        customers.sort_by(|a, b| {
            b.1.problem_rate()
                .partial_cmp(&a.1.problem_rate())
                .unwrap()
        });

        customers.into_iter().take(TOP_N).collect()
    }

    fn high_value_at_risk(&self) -> Vec<(&String, &CustomerData)> {
        let mut customers: Vec<_> = self
            .customers
            .iter()
            .filter(|(_, data)| {
                data.total_orders() >= self.min_orders_threshold
                    && data.problem_rate() > 0.0
                    && data.total_amount > 0.0
            })
            .collect();

        customers.sort_by(|a, b| {
            b.1.total_amount
                .partial_cmp(&a.1.total_amount)
                .unwrap()
        });

        customers.into_iter().take(TOP_N).collect()
    }

    #[cfg(test)]
    fn customer_data(&self, customer: &str) -> Option<&CustomerData> {
        self.customers.get(customer)
    }
}

impl Stat for CustomerRiskProfile {
    fn accept(&mut self, order: &Order) {
        let data = self.customers.entry(order.customer.clone()).or_default();
        data.total_amount += order.amount;

        match order.status {
            OrderStatus::Paid => data.paid_count += 1,
            OrderStatus::Cancelled => data.cancelled_count += 1,
            OrderStatus::Refunded => data.refunded_count += 1,
        }
    }
}

impl Display for CustomerRiskProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- Customer Risk Profile ---")?;
        writeln!(
            f,
            "(Showing customers with {} or more orders)",
            self.min_orders_threshold
        )?;
        writeln!(f)?;

        writeln!(f, "Top {} Highest Risk Customers:", TOP_N)?;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec![
            "Customer",
            "Orders",
            "Problem Rate",
            "Cancel Rate",
            "Refund Rate",
        ]);

        for (name, data) in self.top_risky_customers() {
            let problem_rate = data.problem_rate();
            let color = if problem_rate >= 50.0 {
                Color::Red
            } else if problem_rate >= 25.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            table.add_row(vec![
                Cell::new(name),
                Cell::new(data.total_orders()),
                Cell::new(format!("{:.1}%", problem_rate))
                    .fg(color)
                    .add_attribute(Attribute::Bold),
                Cell::new(format!("{:.1}%", data.cancellation_rate())),
                Cell::new(format!("{:.1}%", data.refund_rate())),
            ]);
        }

        writeln!(f, "{table}")?;

        writeln!(f)?;
        writeln!(f, "High-Value Customers at Risk:")?;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec![
            "Customer",
            "Total Spend",
            "Orders",
            "Problem Rate",
        ]);

        for (name, data) in self.high_value_at_risk() {
            table.add_row(vec![
                name.to_string(),
                format!("${:.2}", data.total_amount),
                data.total_orders().to_string(),
                format!("{:.1}%", data.problem_rate()),
            ]);
        }

        writeln!(f, "{table}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_order(id: u32, customer: &str, amount: f64, status: OrderStatus) -> Order {
        Order {
            id,
            customer: customer.to_string(),
            amount,
            status,
        }
    }

    #[test]
    fn new_profile_is_empty() {
        let profile = CustomerRiskProfile::new();

        assert!(profile.customers.is_empty());
    }

    #[test]
    fn accept_tracks_customer_orders() {
        let mut profile = CustomerRiskProfile::new();

        profile.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        profile.accept(&create_order(2, "John", 200.0, OrderStatus::Cancelled));

        let data = profile.customer_data("John").unwrap();
        assert_eq!(data.paid_count, 1);
        assert_eq!(data.cancelled_count, 1);
        assert_eq!(data.total_orders(), 2);
    }

    #[test]
    fn accept_tracks_refunded_orders() {
        let mut profile = CustomerRiskProfile::new();

        profile.accept(&create_order(1, "Jane", 100.0, OrderStatus::Refunded));

        let data = profile.customer_data("Jane").unwrap();
        assert_eq!(data.refunded_count, 1);
    }

    #[test]
    fn accept_tracks_total_amount() {
        let mut profile = CustomerRiskProfile::new();

        profile.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        profile.accept(&create_order(2, "John", 200.0, OrderStatus::Cancelled));

        let data = profile.customer_data("John").unwrap();
        assert!((data.total_amount - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn problem_rate_calculates_correctly() {
        let mut profile = CustomerRiskProfile::new();

        profile.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        profile.accept(&create_order(2, "John", 100.0, OrderStatus::Cancelled));
        profile.accept(&create_order(3, "John", 100.0, OrderStatus::Refunded));
        profile.accept(&create_order(4, "John", 100.0, OrderStatus::Paid));

        let data = profile.customer_data("John").unwrap();
        // 2 problems out of 4 = 50%
        assert!((data.problem_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cancellation_rate_calculates_correctly() {
        let mut profile = CustomerRiskProfile::new();

        profile.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        profile.accept(&create_order(2, "John", 100.0, OrderStatus::Cancelled));

        let data = profile.customer_data("John").unwrap();
        assert!((data.cancellation_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn refund_rate_calculates_correctly() {
        let mut profile = CustomerRiskProfile::new();

        profile.accept(&create_order(1, "Jane", 100.0, OrderStatus::Paid));
        profile.accept(&create_order(2, "Jane", 100.0, OrderStatus::Refunded));

        let data = profile.customer_data("Jane").unwrap();
        assert!((data.refund_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn top_risky_filters_by_min_orders() {
        let mut profile = CustomerRiskProfile::new();

        // John has only 1 order - should be filtered out
        profile.accept(&create_order(1, "John", 100.0, OrderStatus::Cancelled));

        // Jane has 2 orders - should be included
        profile.accept(&create_order(2, "Jane", 100.0, OrderStatus::Cancelled));
        profile.accept(&create_order(3, "Jane", 100.0, OrderStatus::Cancelled));

        let risky = profile.top_risky_customers();
        assert_eq!(risky.len(), 1);
        assert_eq!(risky[0].0, "Jane");
    }

    #[test]
    fn top_risky_sorts_by_problem_rate() {
        let mut profile = CustomerRiskProfile::new();

        // John: 50% problem rate
        profile.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        profile.accept(&create_order(2, "John", 100.0, OrderStatus::Cancelled));

        // Jane: 100% problem rate
        profile.accept(&create_order(3, "Jane", 100.0, OrderStatus::Cancelled));
        profile.accept(&create_order(4, "Jane", 100.0, OrderStatus::Refunded));

        let risky = profile.top_risky_customers();
        assert_eq!(risky[0].0, "Jane");
        assert_eq!(risky[1].0, "John");
    }

    #[test]
    fn high_value_at_risk_sorts_by_amount() {
        let mut profile = CustomerRiskProfile::new();

        // John: lower amount
        profile.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        profile.accept(&create_order(2, "John", 100.0, OrderStatus::Cancelled));

        // Jane: higher amount
        profile.accept(&create_order(3, "Jane", 500.0, OrderStatus::Paid));
        profile.accept(&create_order(4, "Jane", 500.0, OrderStatus::Refunded));

        let high_value = profile.high_value_at_risk();
        assert_eq!(high_value[0].0, "Jane");
        assert_eq!(high_value[1].0, "John");
    }

    #[test]
    fn high_value_at_risk_excludes_zero_problem_rate() {
        let mut profile = CustomerRiskProfile::new();

        // John: has problems
        profile.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        profile.accept(&create_order(2, "John", 100.0, OrderStatus::Cancelled));

        // Jane: no problems
        profile.accept(&create_order(3, "Jane", 1000.0, OrderStatus::Paid));
        profile.accept(&create_order(4, "Jane", 1000.0, OrderStatus::Paid));

        let high_value = profile.high_value_at_risk();
        assert_eq!(high_value.len(), 1);
        assert_eq!(high_value[0].0, "John");
    }

    #[test]
    fn render_contains_headers() {
        let profile = CustomerRiskProfile::new();
        let output = profile.to_string();

        assert!(output.contains("--- Customer Risk Profile ---"));
        assert!(output.contains("Top 5 Highest Risk Customers"));
        assert!(output.contains("High-Value Customers at Risk"));
    }
}
