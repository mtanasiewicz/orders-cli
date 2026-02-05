use crate::order::Order;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use std::fmt::{Display, Formatter};

const TOP_N: usize = 5;

#[derive(Clone)]
struct OrderSummary {
    id: u32,
    customer: String,
    amount: f64,
}

impl OrderSummary {
    fn from_order(order: &Order) -> Self {
        Self {
            id: order.id,
            customer: order.customer.clone(),
            amount: order.amount,
        }
    }
}

pub struct TopOrders {
    top_highest: Vec<OrderSummary>,
    top_lowest: Vec<OrderSummary>,
}

impl TopOrders {
    pub fn new() -> Self {
        Self {
            top_highest: Vec::with_capacity(TOP_N),
            top_lowest: Vec::with_capacity(TOP_N),
        }
    }

    fn insert_highest(&mut self, order: &Order) {
        if self.top_highest.len() < TOP_N {
            let summary = OrderSummary::from_order(order);

            self.top_highest.push(summary);
            self.top_highest
                .sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
        } else if order.amount > self.top_highest.last().unwrap().amount {
            let summary = OrderSummary::from_order(order);

            self.top_highest.pop();
            self.top_highest.push(summary);
            self.top_highest
                .sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
        }
    }

    fn insert_lowest(&mut self, order: &Order) {
        if self.top_lowest.len() < TOP_N {
            let summary = OrderSummary::from_order(order);

            self.top_lowest.push(summary);
            self.top_lowest
                .sort_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
        } else if order.amount < self.top_lowest.last().unwrap().amount {
            let summary = OrderSummary::from_order(order);

            self.top_lowest.pop();
            self.top_lowest.push(summary);
            self.top_lowest
                .sort_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
        }
    }

    #[cfg(test)]
    fn highest(&self) -> &[OrderSummary] {
        &self.top_highest
    }

    #[cfg(test)]
    fn lowest(&self) -> &[OrderSummary] {
        &self.top_lowest
    }
}

impl TopOrders {
    pub fn accept(&mut self, order: &Order) {
        self.insert_highest(order);
        self.insert_lowest(order);
    }
}

impl Display for TopOrders {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- Top {} Highest Orders ---", TOP_N)?;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec!["Rank", "Order ID", "Customer", "Amount"]);

        for (i, order) in self.top_highest.iter().enumerate() {
            table.add_row(vec![
                (i + 1).to_string(),
                order.id.to_string(),
                order.customer.clone(),
                format!("${:.2}", order.amount),
            ]);
        }

        writeln!(f, "{table}")?;

        writeln!(f)?;
        writeln!(f, "--- Top {} Lowest Orders ---", TOP_N)?;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec!["Rank", "Order ID", "Customer", "Amount"]);

        for (i, order) in self.top_lowest.iter().enumerate() {
            table.add_row(vec![
                (i + 1).to_string(),
                order.id.to_string(),
                order.customer.clone(),
                format!("${:.2}", order.amount),
            ]);
        }

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
    fn new_top_orders_is_empty() {
        let top = TopOrders::new();

        assert!(top.highest().is_empty());
        assert!(top.lowest().is_empty());
    }

    #[test]
    fn accept_adds_to_highest_when_under_capacity() {
        let mut top = TopOrders::new();

        top.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        top.accept(&create_order(2, "Jane", 200.0, OrderStatus::Paid));

        assert_eq!(top.highest().len(), 2);
        assert!((top.highest()[0].amount - 200.0).abs() < f64::EPSILON);
        assert!((top.highest()[1].amount - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn accept_adds_to_lowest_when_under_capacity() {
        let mut top = TopOrders::new();

        top.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        top.accept(&create_order(2, "Jane", 200.0, OrderStatus::Paid));

        assert_eq!(top.lowest().len(), 2);
        assert!((top.lowest()[0].amount - 100.0).abs() < f64::EPSILON);
        assert!((top.lowest()[1].amount - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn highest_maintains_top_n() {
        let mut top = TopOrders::new();

        for i in 1..=10 {
            top.accept(&create_order(i, &format!("Customer{}", i), i as f64 * 10.0, OrderStatus::Paid));
        }

        assert_eq!(top.highest().len(), TOP_N);
        assert!((top.highest()[0].amount - 100.0).abs() < f64::EPSILON);
        assert!((top.highest()[1].amount - 90.0).abs() < f64::EPSILON);
        assert!((top.highest()[2].amount - 80.0).abs() < f64::EPSILON);
        assert!((top.highest()[3].amount - 70.0).abs() < f64::EPSILON);
        assert!((top.highest()[4].amount - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn lowest_maintains_top_n() {
        let mut top = TopOrders::new();

        for i in 1..=10 {
            top.accept(&create_order(i, &format!("Customer{}", i), i as f64 * 10.0, OrderStatus::Paid));
        }

        assert_eq!(top.lowest().len(), TOP_N);
        assert!((top.lowest()[0].amount - 10.0).abs() < f64::EPSILON);
        assert!((top.lowest()[1].amount - 20.0).abs() < f64::EPSILON);
        assert!((top.lowest()[2].amount - 30.0).abs() < f64::EPSILON);
        assert!((top.lowest()[3].amount - 40.0).abs() < f64::EPSILON);
        assert!((top.lowest()[4].amount - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn highest_replaces_when_new_is_higher() {
        let mut top = TopOrders::new();

        for i in 1..=5 {
            top.accept(&create_order(i, &format!("Customer{}", i), i as f64 * 10.0, OrderStatus::Paid));
        }

        // Add a higher one
        top.accept(&create_order(6, "HighCustomer", 1000.0, OrderStatus::Paid));

        assert_eq!(top.highest().len(), TOP_N);
        assert!((top.highest()[0].amount - 1000.0).abs() < f64::EPSILON);
        assert_eq!(top.highest()[0].customer, "HighCustomer");
    }

    #[test]
    fn lowest_replaces_when_new_is_lower() {
        let mut top = TopOrders::new();

        for i in 1..=5 {
            top.accept(&create_order(i, &format!("Customer{}", i), i as f64 * 100.0, OrderStatus::Paid));
        }

        // Add a lower one
        top.accept(&create_order(6, "LowCustomer", 1.0, OrderStatus::Paid));

        assert_eq!(top.lowest().len(), TOP_N);
        assert!((top.lowest()[0].amount - 1.0).abs() < f64::EPSILON);
        assert_eq!(top.lowest()[0].customer, "LowCustomer");
    }

    #[test]
    fn highest_does_not_replace_when_new_is_lower() {
        let mut top = TopOrders::new();

        for i in 1..=5 {
            top.accept(&create_order(i, &format!("Customer{}", i), i as f64 * 100.0, OrderStatus::Paid));
        }

        // Try to add a lower one
        top.accept(&create_order(6, "LowCustomer", 1.0, OrderStatus::Paid));

        // The lowest in top_highest should still be 100
        assert!((top.highest()[4].amount - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn render_contains_headers() {
        let top = TopOrders::new();
        let output = top.to_string();

        assert!(output.contains("--- Top 5 Highest Orders ---"));
        assert!(output.contains("--- Top 5 Lowest Orders ---"));
    }
}
